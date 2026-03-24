use mongodb::bson::doc;
use mongodb::options::IndexOptions;
use mongodb::{Database, IndexModel};

pub async fn init_indexes(db: &Database) -> anyhow::Result<()> {
    let slices = db.collection::<mongodb::bson::Document>("slices");
    slices
        .create_index(
            IndexModel::builder()
                .keys(doc! { "snssai.sst": 1, "snssai.sd": 1, "plmnId.mcc": 1, "plmnId.mnc": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;
    tracing::info!("Created index on slices (snssai, plmnId)");

    let subscriptions = db.collection::<mongodb::bson::Document>("subscriptions");
    subscriptions
        .create_index(
            IndexModel::builder()
                .keys(doc! { "supi": 1, "plmnId.mcc": 1, "plmnId.mnc": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;
    tracing::info!("Created index on subscriptions (supi, plmnId)");

    let policies = db.collection::<mongodb::bson::Document>("policies");
    policies
        .create_index(
            IndexModel::builder()
                .keys(doc! { "snssai.sst": 1, "snssai.sd": 1, "plmnId.mcc": 1, "plmnId.mnc": 1 })
                .build(),
        )
        .await?;
    tracing::info!("Created index on policies (snssai, plmnId)");

    let nsi = db.collection::<mongodb::bson::Document>("nsi");
    nsi.create_index(
        IndexModel::builder()
            .keys(doc! { "nsiId": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build(),
    )
    .await?;
    nsi.create_index(
        IndexModel::builder()
            .keys(doc! { "snssai.sst": 1, "snssai.sd": 1, "plmnId.mcc": 1, "plmnId.mnc": 1 })
            .build(),
    )
    .await?;
    tracing::info!("Created indexes on nsi (nsiId, snssai+plmnId)");

    let snssai_mappings = db.collection::<mongodb::bson::Document>("snssai_mappings");
    snssai_mappings
        .create_index(
            IndexModel::builder()
                .keys(doc! {
                    "servingSnssai.sst": 1,
                    "servingSnssai.sd": 1,
                    "servingPlmn.mcc": 1,
                    "servingPlmn.mnc": 1,
                    "homePlmn.mcc": 1,
                    "homePlmn.mnc": 1,
                })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;
    tracing::info!("Created index on snssai_mappings");

    let nsag_configurations = db.collection::<mongodb::bson::Document>("nsag_configurations");
    nsag_configurations
        .create_index(
            IndexModel::builder()
                .keys(doc! { "nsagId": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;
    tracing::info!("Created index on nsag_configurations (nsagId)");

    let nssrg_configurations = db.collection::<mongodb::bson::Document>("nssrg_configurations");
    nssrg_configurations
        .create_index(
            IndexModel::builder()
                .keys(doc! { "nssrgId": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;
    tracing::info!("Created index on nssrg_configurations (nssrgId)");

    let amf_sets = db.collection::<mongodb::bson::Document>("amf_sets");
    amf_sets
        .create_index(
            IndexModel::builder()
                .keys(doc! { "amfSetId": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;
    tracing::info!("Created index on amf_sets (amfSetId)");

    let amf_service_sets = db.collection::<mongodb::bson::Document>("amf_service_sets");
    amf_service_sets
        .create_index(
            IndexModel::builder()
                .keys(doc! { "amfSetId": 1, "nrfId": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;
    tracing::info!("Created index on amf_service_sets (amfSetId, nrfId)");

    let amf_instances = db.collection::<mongodb::bson::Document>("amf_instances");
    amf_instances
        .create_index(
            IndexModel::builder()
                .keys(doc! { "nfInstanceId": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;
    tracing::info!("Created index on amf_instances (nfInstanceId)");

    let nssai_availability_subscriptions =
        db.collection::<mongodb::bson::Document>("nssai_availability_subscriptions");
    nssai_availability_subscriptions
        .create_index(
            IndexModel::builder()
                .keys(doc! { "subscriptionId": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;
    nssai_availability_subscriptions
        .create_index(
            IndexModel::builder()
                .keys(doc! { "expiry": 1 })
                .build(),
        )
        .await?;
    tracing::info!("Created indexes on nssai_availability_subscriptions");

    tracing::info!("All NSSF database indexes initialized");
    Ok(())
}
