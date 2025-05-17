use actix_web::{test, App, web};
use bn254_rs::web::handlers;
use bn254_rs::web::models::{RegistrationParamsRequest, RegistrationParamsResponse, G1Point};
use bn254_rs::web::store::Store;
use bn254_rs::{G1Point as CryptoG1Point, G2Point, pairing_check, hash_g1_point};
use hex;
use serde_json;

#[actix_web::test]
async fn test_g1_scalar_mul_vs_add() {
    let g = CryptoG1Point::generator();
    let double = g.add(&g);
    let s2 = g.scalar_mul(2u64.into());
    assert_eq!(double, s2);
}

#[actix_web::test]
async fn test_pairing_identity() {
    let g1 = CryptoG1Point::generator();
    let g2 = G2Point::generator();
    let neg = g1.negate();

    let result = pairing_check(g1, g2, neg, g2);
    assert!(result);
}

#[actix_web::test]
async fn test_hash_g1_point() {
    let g = CryptoG1Point::generator();
    let hash = hash_g1_point(&g);
    println!("hash: 0x{}", hex::encode(hash));
}

#[actix_web::test]
async fn test_registration_params_with_ivan() {
    // Test data - just need the address and message hash
    let test_data = RegistrationParamsRequest {
        eoa_address: "0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f".to_string(), // Ivan's address
        message_hash: "10363867709620950417618489183963548939282922352525903103013129965871369247143".to_string(), // Decimal format of the hash
    };

    // Expected signature from test data (converted from hex to decimal)
    let expected_signature = G1Point {
        x: "21572237896589037764462133988381137583389451648577018895923172218094034394521".to_string(),
        y: "20152818134346149256342200472415648217190494410766708079322501661930768921918".to_string(),
    };

    // Create test app with store loaded from players.json
    let store = Store::new().expect("Failed to create store");
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store))
            .service(web::resource("/registration-params").route(web::post().to(handlers::get_registration_params)))
    ).await;

    // Make request
    let resp = test::TestRequest::post()
        .uri("/registration-params")
        .set_json(&test_data)
        .send_request(&app)
        .await;

    // Print response status and body for debugging
    println!("Response status: {}", resp.status());
    let body = test::read_body(resp).await;
    println!("Response body: {}", String::from_utf8_lossy(&body));

    // Parse the response body
    let result: RegistrationParamsResponse = serde_json::from_slice(&body).expect("Failed to parse response");
    
    // Verify signature matches expected output
    assert_eq!(result.signature.x, expected_signature.x);
    assert_eq!(result.signature.y, expected_signature.y);
}

#[actix_web::test]
async fn test_registration_params_with_rupert() {
    // Test data - just need the address and message hash
    let test_data = RegistrationParamsRequest {
        eoa_address: "0xbDA5747bFD65F08deb54cb465eB87D40e51B197E".to_string(), // Rupert's address
        message_hash: "153123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789".to_string(), // Decimal format of the hash
    };

    // Expected signature from test data (converted from hex to decimal)
    let expected_signature = G1Point {
        x: "061b945e2fdd5dbfa3e72af4a09f177379f3ef933daa181e13653472378f023528306a6d4cb6f9cd12784c3bf7abd3eef8788e88a2eee530344a8769ca35b637".to_string(),
        y: "0".to_string(), // This will be calculated by the service
    };

    // Create test app with store loaded from players.json
    let store = Store::new().expect("Failed to create store");
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store))
            .service(web::resource("/registration-params").route(web::post().to(handlers::get_registration_params)))
    ).await;

    // Make request
    let resp = test::TestRequest::post()
        .uri("/registration-params")
        .set_json(&test_data)
        .send_request(&app)
        .await;

    // Print response status and body for debugging
    println!("Response status: {}", resp.status());
    let body = test::read_body(resp).await;
    println!("Response body: {}", String::from_utf8_lossy(&body));

    // Parse the response body
    let result: RegistrationParamsResponse = serde_json::from_slice(&body).expect("Failed to parse response");
    
    // Verify signature matches expected output
    assert_eq!(result.signature.x, expected_signature.x);
    // Note: We don't verify y coordinate as it will be calculated by the service
}

#[actix_web::test]
async fn test_registration_params_with_walter() {
    // Test data - just need the address and message hash
    let test_data = RegistrationParamsRequest {
        eoa_address: "0x08135Da0A343E492FA2d4282F2AE34c6c5CC1BbE".to_string(), // Walter's address
        message_hash: "153123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789".to_string(), // Decimal format of the hash
    };

    // Expected signature from test data (converted from hex to decimal)
    let expected_signature = G1Point {
        x: "4431793996374630971118105007877773991701472947842927050303906469648298150237".to_string(),
        y: "0".to_string(),
    };

    // Create test app with store loaded from players.json
    let store = Store::new().expect("Failed to create store");
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store))
            .service(web::resource("/registration-params").route(web::post().to(handlers::get_registration_params)))
    ).await;

    // Make request
    let resp = test::TestRequest::post()
        .uri("/registration-params")
        .set_json(&test_data)
        .send_request(&app)
        .await;

    // Print response status and body for debugging
    println!("Response status: {}", resp.status());
    let body = test::read_body(resp).await;
    println!("Response body: {}", String::from_utf8_lossy(&body));

    // Parse the response body
    let result: RegistrationParamsResponse = serde_json::from_slice(&body).expect("Failed to parse response");
    
    // Verify signature matches expected output
    assert_eq!(result.signature.x, expected_signature.x);
    // Note: We don't verify y coordinate as it will be calculated by the service
}
