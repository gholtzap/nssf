use mongodb::Database;
use mongodb::IndexModel;
use mongodb::bson::doc;
use mongodb::options::IndexOptions;

pub async fn init_indexes(db: &Database) -> anyhow::Result<()> {
    let unique = IndexOptions::builder().unique(true).build();

    db.collection::<bson::Document>("slices")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "snssai.sst": 1, "snssai.sd": 1, "plmnId.mcc": 1, "plmnId.mnc": 1 })
                .options(unique.clone())
                .build(),
        )
        .await?;

    db.collection::<bson::Document>("subscriptions")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "supi": 1 })
                .options(unique.clone())
                .build(),
        )
        .await?;

    db.collection::<bson::Document>("policies")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "snssai.sst": 1, "snssai.sd": 1 })
                .build(),
        )
        .await?;

    db.collection::<bson::Document>("nsi")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "nsiId": 1 })
                .options(unique.clone())
                .build(),
        )
        .await?;

    db.collection::<bson::Document>("snssai_mappings")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "servingSnssai.sst": 1, "servingSnssai.sd": 1 })
                .build(),
        )
        .await?;

    db.collection::<bson::Document>("nsag_configurations")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "nsagId": 1 })
                .options(unique.clone())
                .build(),
        )
        .await?;

    db.collection::<bson::Document>("nssrg_configurations")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "nssrgId": 1 })
                .options(unique.clone())
                .build(),
        )
        .await?;

    db.collection::<bson::Document>("amf_sets")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "amfSetId": 1 })
                .options(unique.clone())
                .build(),
        )
        .await?;

    db.collection::<bson::Document>("amf_service_sets")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "amfSetId": 1, "amfServiceSetId": 1 })
                .options(unique.clone())
                .build(),
        )
        .await?;

    db.collection::<bson::Document>("amf_instances")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "nfInstanceId": 1 })
                .options(unique.clone())
                .build(),
        )
        .await?;

    db.collection::<bson::Document>("nssai_availability_subscriptions")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "subscriptionId": 1 })
                .options(unique)
                .build(),
        )
        .await?;

    tracing::info!("MongoDB indexes initialized");
    Ok(())
}
