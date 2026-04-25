use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    GenericImage,
};

const TEST_IMG: &str = "viettrann/fundamental";
const TEST_PORT: u16 = 3000;

#[tokio::test]
async fn test_quarly_report() {
    let reports = vec!["quarly", "annually", "history"];
    let container = GenericImage::new(TEST_IMG, std::env::consts::ARCH)
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
