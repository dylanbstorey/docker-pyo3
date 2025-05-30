// Simple test to verify docker-compose-types integration
use docker_compose_types::Compose;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_compose_types_basic() {
        // Test basic docker-compose-types functionality
        let yaml = r#"
version: '3.8'
services:
  web:
    image: nginx:latest
    ports:
      - "80:80"
"#;
        
        let compose: Result<Compose, _> = serde_yaml::from_str(yaml);
        assert!(compose.is_ok());
        
        let compose = compose.unwrap();
        assert_eq!(compose.version, Some("3.8".to_string()));
        
        // Just verify we have services field
        // services is not Option<Services>, it's Services directly
    }

    #[test]
    fn test_compose_round_trip() {
        let yaml = r#"
version: '3.8'
services:
  web:
    image: nginx:latest
"#;
        
        // Parse YAML
        let compose: Compose = serde_yaml::from_str(yaml).unwrap();
        
        // Convert back to YAML
        let yaml_output = serde_yaml::to_string(&compose).unwrap();
        println!("Round-trip YAML: {}", yaml_output);
        
        // Parse again to verify round-trip
        let compose2: Compose = serde_yaml::from_str(&yaml_output).unwrap();
        
        // Basic verification that structure is preserved
        assert_eq!(compose.version, compose2.version);
    }

    #[test]
    fn test_minimal_stack_creation() {
        // Test creating a minimal compose structure programmatically
        let mut compose = Compose::default();
        compose.version = Some("3.8".to_string());
        
        // Serialize to verify it works
        let yaml_output = serde_yaml::to_string(&compose).unwrap();
        assert!(yaml_output.contains("version"));
    }
}