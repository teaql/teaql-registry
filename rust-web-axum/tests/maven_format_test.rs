use nexus_repository_service_core_workspace::format::maven::{
    generate_maven_metadata_xml, parse_maven_path,
};

#[test]
fn test_parse_standard_release_jar() {
    let path = "/org/apache/commons/commons-lang3/3.12.0/commons-lang3-3.12.0.jar";
    let coords = parse_maven_path(path).expect("Failed to parse standard release jar");

    assert_eq!(coords.group_id, "org.apache.commons");
    assert_eq!(coords.artifact_id, "commons-lang3");
    assert_eq!(coords.version, "3.12.0");
    assert_eq!(coords.file_name, "commons-lang3-3.12.0.jar");
    assert_eq!(coords.extension, "jar");
    assert!(!coords.is_snapshot);
    assert!(!coords.is_metadata);
}

#[test]
fn test_parse_standard_pom_and_sources() {
    let pom_path = "/com/google/guava/guava/31.1-jre/guava-31.1-jre.pom";
    let pom_coords = parse_maven_path(pom_path).expect("Failed to parse pom path");
    assert_eq!(pom_coords.group_id, "com.google.guava");
    assert_eq!(pom_coords.artifact_id, "guava");
    assert_eq!(pom_coords.version, "31.1-jre");
    assert_eq!(pom_coords.extension, "pom");
    assert!(!pom_coords.is_snapshot);

    let src_path = "/com/google/guava/guava/31.1-jre/guava-31.1-jre-sources.jar";
    let src_coords = parse_maven_path(src_path).expect("Failed to parse sources jar");
    assert_eq!(src_coords.group_id, "com.google.guava");
    assert_eq!(src_coords.artifact_id, "guava");
    assert_eq!(src_coords.version, "31.1-jre");
    assert_eq!(src_coords.extension, "jar");
    assert!(!src_coords.is_snapshot);
}

#[test]
fn test_parse_snapshot_artifact() {
    let path = "/com/example/demo/1.0.0-SNAPSHOT/demo-1.0.0-SNAPSHOT.jar";
    let coords = parse_maven_path(path).expect("Failed to parse snapshot artifact");

    assert_eq!(coords.group_id, "com.example");
    assert_eq!(coords.artifact_id, "demo");
    assert_eq!(coords.version, "1.0.0-SNAPSHOT");
    assert_eq!(coords.file_name, "demo-1.0.0-SNAPSHOT.jar");
    assert_eq!(coords.extension, "jar");
    assert!(coords.is_snapshot);
    assert!(!coords.is_metadata);
}

#[test]
fn test_parse_maven_metadata() {
    let root_meta = "/org/apache/commons/commons-lang3/maven-metadata.xml";
    let coords = parse_maven_path(root_meta).expect("Failed to parse metadata");

    assert_eq!(coords.group_id, "org.apache.commons");
    assert_eq!(coords.artifact_id, "commons-lang3");
    assert_eq!(coords.file_name, "maven-metadata.xml");
    assert_eq!(coords.extension, "xml");
    assert!(coords.is_metadata);

    let sha1_meta = "/org/apache/commons/commons-lang3/maven-metadata.xml.sha1";
    let sha1_coords = parse_maven_path(sha1_meta).expect("Failed to parse sha1 metadata");
    assert_eq!(sha1_coords.group_id, "org.apache.commons");
    assert_eq!(sha1_coords.artifact_id, "commons-lang3");
    assert_eq!(sha1_coords.file_name, "maven-metadata.xml.sha1");
    assert!(sha1_coords.is_metadata);
}

#[test]
fn test_parse_invalid_maven_paths() {
    assert_eq!(parse_maven_path(""), None);
    assert_eq!(parse_maven_path("/"), None);
    assert_eq!(parse_maven_path("/com"), None);
    assert_eq!(parse_maven_path("/com/example"), None);
    assert_eq!(parse_maven_path("demo.jar"), None);
}

#[test]
fn test_generate_maven_metadata_xml() {
    let versions = vec![
        "1.0.0".to_string(),
        "1.1.0".to_string(),
        "2.0.0-SNAPSHOT".to_string(),
    ];
    let xml = generate_maven_metadata_xml("com.example", "my-app", &versions);

    assert!(xml.contains("<groupId>com.example</groupId>"));
    assert!(xml.contains("<artifactId>my-app</artifactId>"));
    assert!(xml.contains("<latest>2.0.0-SNAPSHOT</latest>"));
    assert!(xml.contains("<release>1.1.0</release>"));
    assert!(xml.contains("<version>1.0.0</version>"));
    assert!(xml.contains("<version>1.1.0</version>"));
    assert!(xml.contains("<version>2.0.0-SNAPSHOT</version>"));
    assert!(xml.contains("<lastUpdated>"));
}
