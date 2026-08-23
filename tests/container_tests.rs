mod common;
use common::*;
use fundamental::common::AppState;
use fundamental::financial_stmt::sec_client::SecClient;
use tokio::sync::Mutex;

use std::sync::Arc;
use std::{collections::HashMap, time::Duration};

use axum::{http::StatusCode, Router};
use fundamental::db::{dragonfly_cache::DragonFlyCache, DataManager};
use fundamental::jobs;
use fundamental::processor::Processor;
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    GenericImage,
};

const TEST_IMG: &str = "viettrann/fundamental";
const TEST_PORT: u16 = 8001;

#[tokio::test]
#[ignore]
async fn test_period_reports() {
    let reports = vec!["quarly", "annually", "history"];
    let mut container_arch = std::env::consts::ARCH;
    container_arch = if container_arch == "aarch64" {
        "arm64"
    } else {
        container_arch
    };

    let container = GenericImage::new(TEST_IMG, container_arch)
        .with_exposed_port(TEST_PORT.tcp())
        .with_wait_for(WaitFor::seconds(1))
        .start()
        .await
        .unwrap();
    let host = container.get_host().await.unwrap();
    let host_port = container.get_host_port_ipv4(TEST_PORT).await.unwrap();

    // Test in different report periods
    for report in reports {
        let resp = reqwest::get(format!("http://{host}:{host_port}/COIN/{report}"))
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        assert_ne!(resp.text().await.unwrap(), "");
    }
}

#[tokio::test]
async fn test_cache_db() {
    let (_cache_db_container, host, host_port) = init_cache_db().await;
    let timeout_sec: i64 = 5;
    let mut db = DragonFlyCache::init(&format!("redis://{host}:{host_port}"), timeout_sec)
        .await
        .unwrap();

    let set_val = HashMap::from([(String::from("test"), 10.0)]);
    let key = String::from("Foo");
    db.set(key.clone(), set_val.clone()).await;
    let get_val = db.get(key.clone()).await.unwrap();
    assert_eq!(get_val, set_val);

    // Test timeout
    tokio::time::sleep(Duration::from_secs((timeout_sec + 1) as u64)).await;
    let expire_val = db.get(key.clone()).await.unwrap();
    assert!(
        expire_val.is_empty(),
        "Faile: test key {} should be empty after timeout",
        key
    );
    assert!(
        db.is_empty().await.unwrap(),
        "Failed: Cache db should be empty"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_avg_ratios_requests_handler() {
    let (_cache_db_container, host, host_port) = init_cache_db().await;
    let timeout_sec: i64 = 60 * 60;
    let db = DragonFlyCache::init(&format!("redis://{host}:{host_port}"), timeout_sec)
        .await
        .unwrap();
    let shared_db = Arc::new(Mutex::new(db));
    let proc = Arc::new(Mutex::new(Processor::default()));

    let app_state = AppState {
        sec_client: Arc::new(Mutex::new(SecClient::new(String::from("")))),
        proc: proc.clone(),
        db: shared_db.clone(),
    };
    {
        let mut proc_lock = proc.lock().await;
        let mut db_lock = shared_db.lock().await;
        jobs::job_calculate_industry_ratio_average(&mut proc_lock, &mut *db_lock).await;
    }
    let app = build_app(Some(app_state)).await;
    assert_ratios_ok(app.clone(), "/COIN/ratios").await;
    assert_ratios_ok(app, "/NVDA/ratios").await;
}

async fn assert_ratios_ok(app: Router, uri: &str) {
    let (status, body) = get(app, uri).await;
    assert_eq!(status, StatusCode::OK, "response: {:?}", body);
    let ratios = parse_json(&body);
    assert_ne!(ratios["current_ratios"], 0.0);
    assert_ne!(ratios["quick_ratio"], 0.0);
    assert_ne!(ratios["debt_ratio"], 0.0);
    assert_ne!(ratios["equity_ratio"], 0.0);
}
