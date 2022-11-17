use std::collections::HashSet;

use anyhow::{bail, Result};
use chbs::prelude::WordProvider;
use chbs::word::{WordList, WordSampler};
use mongodb::bson::{doc, Bson};
use mongodb::options::{IndexOptions, UpdateModifications};
use mongodb::{bson, Collection, IndexModel};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum IM {
    QQ,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Group {
    pub im: IM,
    pub id: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct QQMessageHandle {
    pub group: i64,
    pub seqs: Vec<i32>,
    pub rands: Vec<i32>,
}

impl Group {
    pub fn from_qq(group_code: i64) -> Self {
        Self {
            im: IM::QQ,
            id: group_code.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    pub name: String,
    pub groups: HashSet<Group>,
}

#[derive(Debug, Clone)]
pub struct DB {
    pub clusters: Collection<Cluster>,
}

impl DB {
    pub async fn connect(uri: &str, db: &str) -> Result<Self> {
        let client = mongodb::Client::with_uri_str(uri).await?;
        let db = client.database(db);
        let clusters = db.collection("clusters");
        clusters
            .create_index(
                IndexModel::builder()
                    .keys(doc! {
                        "name": 1
                    })
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
                None,
            )
            .await?;
        Ok(Self { clusters })
    }
    pub async fn new_cluster(&self) -> Result<String> {
        static SAMPLER: Lazy<WordSampler> = Lazy::new(|| WordList::builtin_eff_short().sampler());
        let name = SAMPLER.word();
        let cluster = Cluster {
            name: name.clone(),
            groups: Default::default(),
        };
        self.clusters.insert_one(cluster, None).await?;
        Ok(name)
    }
    pub async fn clusters(&self) -> Result<impl Iterator<Item = String>> {
        Ok(self
            .clusters
            .distinct("name", None, None)
            .await?
            .into_iter()
            .map(|doc| {
                if let Bson::String(s) = doc {
                    s
                } else {
                    unreachable!()
                }
            }))
    }
    pub async fn join(&self, cluster: &str, group: &Group) -> Result<()> {
        let group = bson::to_document(group)?;
        let result = self
            .clusters
            .update_one(
                doc! {
                    "name": {
                        "$eq": cluster
                    }
                },
                UpdateModifications::Document(doc! {
                    "$addToSet": {
                        "groups": group
                    }
                }),
                None,
            )
            .await?;
        if result.modified_count == 0 {
            bail!("No cluster modified.");
        }
        Ok(())
    }
    pub async fn forward_targets(&self, group: &Group) -> Result<Vec<Group>> {
        #[derive(Debug, Deserialize)]
        struct Targets {
            targets: Vec<Group>,
        }
        let group = bson::to_document(group)?;
        let mut cursor = self
            .clusters
            .aggregate(
                [
                    doc! {
                        "$match": doc! {
                            "groups": doc! {
                                "$all": [
                                    &group
                                ]
                            }
                        }
                    },
                    doc! {
                        "$unwind": doc! {
                            "path": "$groups"
                        }
                    },
                    doc! {
                        "$match": doc! {
                            "groups": doc! {
                                "$ne": group
                            }
                        }
                    },
                    doc! {
                        "$group": doc! {
                            "_id": Bson::Null,
                            "targets": doc! {
                                "$addToSet": "$groups"
                            }
                        }
                    },
                ],
                None,
            )
            .await?
            .with_type::<Targets>();
        Ok(if cursor.advance().await? {
            let targets = cursor.deserialize_current()?;
            targets.targets
        } else {
            vec![]
        })
    }
}
