use nexus_repository_service_core_workspace::format::docker::{
    compute_sha256_digest, is_valid_digest, parse_docker_path, DockerDescriptor, DockerManifestV2,
    DockerPath, DOCKER_CONFIG_JSON_MEDIA_TYPE, DOCKER_LAYER_GZIP_MEDIA_TYPE,
    DOCKER_MANIFEST_V2_MEDIA_TYPE,
};

#[test]
fn test_compute_sha256_digest_and_validation() {
    let payload = b"Hello Docker Registry OCI";
    let digest = compute_sha256_digest(payload);

    assert!(digest.starts_with("sha256:"));
    assert_eq!(digest.len(), 71); // "sha256:" (7) + 64 hex chars
    assert!(is_valid_digest(&digest));
    assert!(!is_valid_digest("md5:123456"));
    assert!(!is_valid_digest("sha256:invalid-hex!"));
}

#[test]
fn test_parse_docker_paths() {
    assert_eq!(parse_docker_path("/v2/"), Some(DockerPath::BasePing));
    assert_eq!(parse_docker_path("/v2"), Some(DockerPath::BasePing));

    assert_eq!(
        parse_docker_path("/v2/ubuntu/tags/list"),
        Some(DockerPath::TagsList {
            name: "ubuntu".to_string()
        })
    );
    assert_eq!(
        parse_docker_path("/v2/org/team/app/tags/list"),
        Some(DockerPath::TagsList {
            name: "org/team/app".to_string()
        })
    );

    assert_eq!(
        parse_docker_path("/v2/ubuntu/blobs/uploads/"),
        Some(DockerPath::BlobsUploadInit {
            name: "ubuntu".to_string()
        })
    );

    assert_eq!(
        parse_docker_path("/v2/ubuntu/blobs/uploads/1234-uuid"),
        Some(DockerPath::BlobsUploadChunk {
            name: "ubuntu".to_string(),
            uuid: "1234-uuid".to_string()
        })
    );

    assert_eq!(
        parse_docker_path("/v2/ubuntu/blobs/sha256:abc1234567890"),
        Some(DockerPath::Blob {
            name: "ubuntu".to_string(),
            digest: "sha256:abc1234567890".to_string()
        })
    );

    assert_eq!(
        parse_docker_path("/v2/ubuntu/manifests/latest"),
        Some(DockerPath::Manifest {
            name: "ubuntu".to_string(),
            reference: "latest".to_string()
        })
    );

    assert_eq!(
        parse_docker_path("/v2/org/service/manifests/sha256:feedbeef"),
        Some(DockerPath::Manifest {
            name: "org/service".to_string(),
            reference: "sha256:feedbeef".to_string()
        })
    );
}

#[test]
fn test_docker_manifest_serde() {
    let manifest = DockerManifestV2 {
        schema_version: 2,
        media_type: DOCKER_MANIFEST_V2_MEDIA_TYPE.to_string(),
        config: DockerDescriptor {
            media_type: DOCKER_CONFIG_JSON_MEDIA_TYPE.to_string(),
            size: 1420,
            digest: "sha256:b5b2b2c507a0944348e0303114d8d93aaaa8ceca7116ff211d704d00db459c4f"
                .to_string(),
            urls: None,
        },
        layers: vec![DockerDescriptor {
            media_type: DOCKER_LAYER_GZIP_MEDIA_TYPE.to_string(),
            size: 524288,
            digest: "sha256:c54374f004689e3241189443494ab3ec4572dd811568913b8606c483a936a2de"
                .to_string(),
            urls: None,
        }],
    };

    let json_str = serde_json::to_string(&manifest).expect("Serialize manifest failed");
    assert!(json_str.contains("schemaVersion"));
    assert!(json_str.contains("application/vnd.docker.distribution.manifest.v2+json"));

    let deserialized: DockerManifestV2 =
        serde_json::from_str(&json_str).expect("Deserialize manifest failed");
    assert_eq!(deserialized.schema_version, 2);
    assert_eq!(deserialized.layers.len(), 1);
    assert_eq!(
        deserialized.layers[0].digest,
        "sha256:c54374f004689e3241189443494ab3ec4572dd811568913b8606c483a936a2de"
    );
}
