//! Jito Client Unit Tests
//! Tests client creation, URL handling, and configuration

use jito_bundler::JitoClient;

/// Test: Create Jito client with custom URL
#[test]
fn test_create_jito_client() {
    let client = JitoClient::new("https://custom.block-engine.jito.wtf".to_string()).unwrap();
    
    assert_eq!(client.block_engine_url(), "https://custom.block-engine.jito.wtf");
}

/// Test: Create devnet client
#[test]
fn test_devnet_client() {
    let client = JitoClient::devnet().unwrap();
    
    assert!(client.block_engine_url().contains("devnet"));
    assert!(client.block_engine_url().contains("block-engine"));
}

/// Test: Create mainnet client
#[test]
fn test_mainnet_client() {
    let client = JitoClient::mainnet().unwrap();
    
    assert!(client.block_engine_url().contains("mainnet"));
    assert!(client.block_engine_url().contains("block-engine"));
}

/// Test: Devnet and mainnet have different URLs
#[test]
fn test_devnet_mainnet_different() {
    let devnet = JitoClient::devnet().unwrap();
    let mainnet = JitoClient::mainnet().unwrap();
    
    assert_ne!(devnet.block_engine_url(), mainnet.block_engine_url());
}

/// Test: Client URL getter
#[test]
fn test_client_url_getter() {
    let url = "https://test.jito.wtf";
    let client = JitoClient::new(url.to_string()).unwrap();
    
    assert_eq!(client.block_engine_url(), url);
}

/// Test: Multiple client instances
#[test]
fn test_multiple_clients() {
    let client1 = JitoClient::new("https://url1.jito.wtf".to_string()).unwrap();
    let client2 = JitoClient::new("https://url2.jito.wtf".to_string()).unwrap();
    
    assert_ne!(client1.block_engine_url(), client2.block_engine_url());
}

/// Test: Client with empty URL
#[test]
fn test_client_empty_url() {
    let client = JitoClient::new(String::new()).unwrap();
    
    assert_eq!(client.block_engine_url(), "");
}

/// Test: Client with very long URL
#[test]
fn test_client_long_url() {
    let long_url = format!("https://{}.jito.wtf", "a".repeat(100));
    let client = JitoClient::new(long_url.clone()).unwrap();
    
    assert_eq!(client.block_engine_url(), long_url);
}

/// Test: Devnet URL format
#[test]
fn test_devnet_url_format() {
    let client = JitoClient::devnet().unwrap();
    let url = client.block_engine_url();
    
    assert!(url.starts_with("https://"));
    assert!(url.ends_with(".jito.wtf"));
}

/// Test: Mainnet URL format
#[test]
fn test_mainnet_url_format() {
    let client = JitoClient::mainnet().unwrap();
    let url = client.block_engine_url();
    
    assert!(url.starts_with("https://"));
    assert!(url.ends_with(".jito.wtf"));
}

/// Test: Client with localhost URL
#[test]
fn test_client_localhost() {
    let client = JitoClient::new("http://localhost:8080".to_string()).unwrap();
    
    assert_eq!(client.block_engine_url(), "http://localhost:8080");
}

/// Test: Client with IP address
#[test]
fn test_client_ip_address() {
    let client = JitoClient::new("http://192.168.1.1:8080".to_string()).unwrap();
    
    assert_eq!(client.block_engine_url(), "http://192.168.1.1:8080");
}

/// Test: Client URL with trailing slash
#[test]
fn test_client_trailing_slash() {
    let client = JitoClient::new("https://test.jito.wtf/".to_string()).unwrap();
    
    assert_eq!(client.block_engine_url(), "https://test.jito.wtf/");
}

/// Test: Client URL with path
#[test]
fn test_client_url_with_path() {
    let client = JitoClient::new("https://test.jito.wtf/api/v1".to_string()).unwrap();
    
    assert_eq!(client.block_engine_url(), "https://test.jito.wtf/api/v1");
}

/// Test: Client URL with port
#[test]
fn test_client_url_with_port() {
    let client = JitoClient::new("https://test.jito.wtf:443".to_string()).unwrap();
    
    assert_eq!(client.block_engine_url(), "https://test.jito.wtf:443");
}

/// Test: Client URL with query parameters
#[test]
fn test_client_url_with_query() {
    let client = JitoClient::new("https://test.jito.wtf?key=value".to_string()).unwrap();
    
    assert_eq!(client.block_engine_url(), "https://test.jito.wtf?key=value");
}

/// Test: Create many clients
#[test]
fn test_create_many_clients() {
    let clients: Vec<JitoClient> = (0..100)
        .map(|i| JitoClient::new(format!("https://url{}.jito.wtf", i)).unwrap())
        .collect();
    
    assert_eq!(clients.len(), 100);
}

/// Test: Devnet client is consistent
#[test]
fn test_devnet_consistency() {
    let client1 = JitoClient::devnet().unwrap();
    let client2 = JitoClient::devnet().unwrap();
    
    assert_eq!(client1.block_engine_url(), client2.block_engine_url());
}

/// Test: Mainnet client is consistent
#[test]
fn test_mainnet_consistency() {
    let client1 = JitoClient::mainnet().unwrap();
    let client2 = JitoClient::mainnet().unwrap();
    
    assert_eq!(client1.block_engine_url(), client2.block_engine_url());
}

/// Test: Client URL without protocol
#[test]
fn test_client_no_protocol() {
    let client = JitoClient::new("test.jito.wtf".to_string()).unwrap();
    
    assert_eq!(client.block_engine_url(), "test.jito.wtf");
}

/// Test: Client with custom subdomain
#[test]
fn test_client_custom_subdomain() {
    let subdomains = vec!["frankfurt", "amsterdam", "tokyo", "ny"];
    
    for subdomain in subdomains {
        let client = JitoClient::new(format!("https://{}.block-engine.jito.wtf", subdomain)).unwrap();
        assert!(client.block_engine_url().contains(subdomain));
    }
}

/// Test: Client URL comparison
#[test]
fn test_url_comparison() {
    let client = JitoClient::new("https://test.jito.wtf".to_string()).unwrap();
    
    assert_eq!(client.block_engine_url(), "https://test.jito.wtf");
    assert_ne!(client.block_engine_url(), "https://other.jito.wtf");
}

/// Test: Client with special characters in URL
#[test]
fn test_client_special_chars() {
    let client = JitoClient::new("https://test-server_1.jito.wtf".to_string()).unwrap();
    
    assert_eq!(client.block_engine_url(), "https://test-server_1.jito.wtf");
}

/// Test: Client with unicode in URL
#[test]
fn test_client_unicode_url() {
    let client = JitoClient::new("https://тест.jito.wtf".to_string()).unwrap();
    
    assert_eq!(client.block_engine_url(), "https://тест.jito.wtf");
}

/// Test: Sequential client creation
#[test]
fn test_sequential_creation() {
    for i in 0..10 {
        let client = JitoClient::new(format!("https://url{}.jito.wtf", i)).unwrap();
        assert!(client.block_engine_url().contains(&i.to_string()));
    }
}

/// Test: Client URL length
#[test]
fn test_url_length() {
    let short_url = "http://a.b";
    let long_url = format!("https://{}.com", "x".repeat(200));
    
    let client1 = JitoClient::new(short_url.to_string()).unwrap();
    let client2 = JitoClient::new(long_url.clone()).unwrap();
    
    assert_eq!(client1.block_engine_url().len(), short_url.len());
    assert_eq!(client2.block_engine_url().len(), long_url.len());
}

/// Test: Devnet URL structure
#[test]
fn test_devnet_url_structure() {
    let client = JitoClient::devnet().unwrap();
    let url = client.block_engine_url();
    
    // Should have protocol, subdomain, domain
    assert!(url.contains("://"));
    assert!(url.contains("."));
}

/// Test: Mainnet URL structure
#[test]
fn test_mainnet_url_structure() {
    let client = JitoClient::mainnet().unwrap();
    let url = client.block_engine_url();
    
    // Should have protocol, subdomain, domain
    assert!(url.contains("://"));
    assert!(url.contains("."));
}

/// Test: Client with region-specific URLs
#[test]
fn test_region_specific_urls() {
    let regions = vec![
        ("frankfurt", "eu"),
        ("ny", "us"),
        ("tokyo", "asia"),
    ];
    
    for (region, _continent) in regions {
        let client = JitoClient::new(format!("https://{}.jito.wtf", region)).unwrap();
        assert!(client.block_engine_url().contains(region));
    }
}
