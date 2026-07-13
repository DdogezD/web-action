use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Result of a policy check for an action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyResult {
    /// Action is allowed.
    Allow,
    /// Action is blocked with the given reason.
    Deny(String),
    /// Action requires confirmation before proceeding.
    RequiresConfirmation,
}

/// Policy configuration loaded from a JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionPolicy {
    #[serde(skip)]
    path: PathBuf,
    #[serde(default)]
    default: Option<String>,
    #[serde(default)]
    allow: Option<Vec<String>>,
    #[serde(default)]
    deny: Option<Vec<String>>,
    #[serde(default)]
    confirm: Option<Vec<String>>,
}

/// Confirmation categories parsed from WEB_ACTION_CONFIRM_ACTIONS.
#[derive(Debug, Clone)]
pub struct ConfirmActions {
    pub categories: HashSet<String>,
}

impl ConfirmActions {
    pub fn from_env() -> Option<Self> {
        let val = env::var("WEB_ACTION_CONFIRM_ACTIONS").ok()?;
        if val.is_empty() {
            return None;
        }
        let categories: HashSet<String> = val
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        if categories.is_empty() {
            None
        } else {
            Some(Self { categories })
        }
    }

    pub fn requires_confirmation(&self, action: &str) -> bool {
        self.categories.contains(action)
    }
}

fn validate_no_null_fields(contents: &str) -> Result<(), String> {
    let value: Value =
        serde_json::from_str(contents).map_err(|e| format!("Invalid policy JSON: {}", e))?;
    let object = value
        .as_object()
        .ok_or_else(|| "Invalid policy JSON: expected an object".to_string())?;

    for field in ["default", "allow", "deny", "confirm"] {
        if object.get(field).is_some_and(Value::is_null) {
            return Err(format!("Invalid policy JSON: '{}' cannot be null", field));
        }
    }

    Ok(())
}

impl ActionPolicy {
    /// Load policy from a JSON file at the given path.
    pub fn load(path: &str) -> Result<Self, String> {
        let path_buf = PathBuf::from(path);
        let contents = fs::read_to_string(&path_buf)
            .map_err(|e| format!("Failed to read policy file: {}", e))?;
        validate_no_null_fields(&contents)?;
        let mut policy: ActionPolicy =
            serde_json::from_str(&contents).map_err(|e| format!("Invalid policy JSON: {}", e))?;
        policy.validate()?;
        policy.path = path_buf;
        Ok(policy)
    }

    /// Load a configured action policy, if either supported environment variable is set.
    ///
    /// An explicit but unreadable or invalid policy is an error: treating it as
    /// an absent policy would silently disable the configured security boundary.
    pub fn load_from_env() -> Result<Option<Self>, String> {
        let path = match env::var("WEB_ACTION_ACTION_POLICY") {
            Ok(path) => path,
            Err(env::VarError::NotPresent) => match env::var("WEB_ACTION_POLICY") {
                Ok(path) => path,
                Err(env::VarError::NotPresent) => return Ok(None),
                Err(e) => return Err(format!("Invalid WEB_ACTION_POLICY value: {}", e)),
            },
            Err(e) => return Err(format!("Invalid WEB_ACTION_ACTION_POLICY value: {}", e)),
        };

        Self::load(&path)
            .map(Some)
            .map_err(|e| format!("Failed to load action policy '{}': {}", path, e))
    }

    fn validate(&self) -> Result<(), String> {
        if let Some(default) = self.default.as_deref() {
            if !matches!(default.to_ascii_lowercase().as_str(), "allow" | "deny") {
                return Err(format!(
                    "Invalid policy default '{}': use allow or deny",
                    default
                ));
            }
        }

        for (field, actions) in [
            ("allow", self.allow.as_deref()),
            ("deny", self.deny.as_deref()),
            ("confirm", self.confirm.as_deref()),
        ] {
            if actions.is_some_and(|actions| {
                actions
                    .iter()
                    .any(|action| action.is_empty() || action != action.trim())
            }) {
                return Err(format!(
                    "Policy '{}' cannot contain empty or whitespace-padded actions",
                    field
                ));
            }
        }

        Ok(())
    }

    /// Check whether an action is allowed, denied, or requires confirmation.
    pub fn check(&self, action: &str) -> PolicyResult {
        if let Some(deny) = &self.deny {
            if deny.iter().any(|a| a == action) {
                return PolicyResult::Deny(format!("Action '{}' is denied by policy", action));
            }
        }

        if let Some(confirm) = &self.confirm {
            if confirm.iter().any(|a| a == action) {
                return PolicyResult::RequiresConfirmation;
            }
        }

        if self
            .allow
            .as_ref()
            .is_some_and(|allow| allow.iter().any(|allowed| allowed == action))
        {
            return PolicyResult::Allow;
        }

        if self
            .default
            .as_deref()
            .is_some_and(|default| default.eq_ignore_ascii_case("deny"))
        {
            return PolicyResult::Deny(format!(
                "Action '{}' denied: default policy is deny",
                action
            ));
        }

        if self
            .allow
            .as_ref()
            .is_some_and(|allow| !allow.is_empty())
        {
            return PolicyResult::Deny(format!(
                "Action '{}' is not in the allow list",
                action
            ));
        }

        PolicyResult::Allow
    }

    /// Reload policy from the file and atomically replace this policy on success.
    pub fn reload(&mut self) -> Result<(), String> {
        let path = self.path.to_string_lossy().to_string();
        let policy = Self::load(&path)
            .map_err(|e| format!("Failed to reload action policy '{}': {}", path, e))?;
        *self = policy;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::EnvGuard;

    #[test]
    fn test_policy_allow_whitelist() {
        let json = r#"{"allow": ["click", "type"], "deny": [], "confirm": []}"#;
        let policy: ActionPolicy = serde_json::from_str(json).unwrap();
        assert_eq!(policy.check("click"), PolicyResult::Allow);
        assert_eq!(policy.check("type"), PolicyResult::Allow);
        assert!(matches!(policy.check("navigate"), PolicyResult::Deny(_)));
    }

    #[test]
    fn test_policy_deny() {
        let json = r#"{"allow": [], "deny": ["delete"], "confirm": []}"#;
        let policy: ActionPolicy = serde_json::from_str(json).unwrap();
        assert!(matches!(policy.check("delete"), PolicyResult::Deny(_)));
    }

    #[test]
    fn test_policy_confirm() {
        let json = r#"{"allow": [], "deny": [], "confirm": ["submit"]}"#;
        let policy: ActionPolicy = serde_json::from_str(json).unwrap();
        assert_eq!(policy.check("submit"), PolicyResult::RequiresConfirmation);
    }

    #[test]
    fn test_policy_deny_takes_precedence() {
        let json = r#"{"allow": ["danger"], "deny": ["danger"], "confirm": []}"#;
        let policy: ActionPolicy = serde_json::from_str(json).unwrap();
        assert!(matches!(policy.check("danger"), PolicyResult::Deny(_)));
    }

    #[test]
    fn test_policy_confirm_takes_precedence_over_allow() {
        let json = r#"{"allow": ["submit"], "deny": [], "confirm": ["submit"]}"#;
        let policy: ActionPolicy = serde_json::from_str(json).unwrap();
        assert_eq!(policy.check("submit"), PolicyResult::RequiresConfirmation);
    }

    #[test]
    fn test_policy_empty_allow_allows_all() {
        let json = r#"{"allow": [], "deny": [], "confirm": []}"#;
        let policy: ActionPolicy = serde_json::from_str(json).unwrap();
        assert_eq!(policy.check("anything"), PolicyResult::Allow);
    }

    #[test]
    fn test_policy_missing_allow_allows_all() {
        let json = r#"{"deny": []}"#;
        let policy: ActionPolicy = serde_json::from_str(json).unwrap();
        assert_eq!(policy.check("anything"), PolicyResult::Allow);
    }

    #[test]
    fn test_policy_default_allow() {
        let json = r#"{"default": "allow", "deny": ["navigate"]}"#;
        let policy: ActionPolicy = serde_json::from_str(json).unwrap();
        assert_eq!(policy.check("click"), PolicyResult::Allow);
        assert!(matches!(policy.check("navigate"), PolicyResult::Deny(_)));
    }

    #[test]
    fn test_policy_default_deny() {
        let json = r#"{"default": "deny", "allow": ["click"]}"#;
        let policy: ActionPolicy = serde_json::from_str(json).unwrap();
        assert_eq!(policy.check("click"), PolicyResult::Allow);
        assert!(matches!(policy.check("navigate"), PolicyResult::Deny(_)));
    }

    #[test]
    fn test_policy_default_deny_with_empty_allow_denies_unlisted_actions() {
        let policy: ActionPolicy = serde_json::from_str(
            r#"{"default":"deny","allow":[],"deny":[],"confirm":[]}"#,
        )
        .unwrap();
        assert!(matches!(policy.check("navigate"), PolicyResult::Deny(_)));
    }

    #[test]
    fn test_confirm_actions_from_env() {
        let _guard = EnvGuard::new(&["WEB_ACTION_CONFIRM_ACTIONS"]);
        _guard.set("WEB_ACTION_CONFIRM_ACTIONS", "navigate,click,fill");
        let ca = ConfirmActions::from_env().unwrap();
        assert!(ca.requires_confirmation("navigate"));
        assert!(ca.requires_confirmation("click"));
        assert!(ca.requires_confirmation("fill"));
        assert!(!ca.requires_confirmation("screenshot"));
    }

    #[test]
    fn test_load_from_env_returns_none_when_unconfigured() {
        let guard = EnvGuard::new(&["WEB_ACTION_ACTION_POLICY", "WEB_ACTION_POLICY"]);
        guard.remove("WEB_ACTION_ACTION_POLICY");
        guard.remove("WEB_ACTION_POLICY");

        assert!(ActionPolicy::load_from_env().unwrap().is_none());
    }

    #[test]
    fn test_load_from_env_prefers_modern_variable() {
        let guard = EnvGuard::new(&["WEB_ACTION_ACTION_POLICY", "WEB_ACTION_POLICY"]);
        let dir = tempfile::tempdir().unwrap();
        let modern = dir.path().join("modern.json");
        let legacy = dir.path().join("legacy.json");
        fs::write(&modern, r#"{"deny":["navigate"]}"#).unwrap();
        fs::write(&legacy, r#"{"deny":["click"]}"#).unwrap();
        guard.set("WEB_ACTION_ACTION_POLICY", modern.to_str().unwrap());
        guard.set("WEB_ACTION_POLICY", legacy.to_str().unwrap());

        let policy = ActionPolicy::load_from_env().unwrap().unwrap();
        assert!(matches!(policy.check("navigate"), PolicyResult::Deny(_)));
        assert_eq!(policy.check("click"), PolicyResult::Allow);
    }

    #[test]
    fn test_invalid_modern_policy_does_not_fall_back_to_legacy() {
        let guard = EnvGuard::new(&["WEB_ACTION_ACTION_POLICY", "WEB_ACTION_POLICY"]);
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("legacy.json");
        fs::write(&legacy, r#"{"deny":["navigate"]}"#).unwrap();
        guard.set(
            "WEB_ACTION_ACTION_POLICY",
            dir.path().join("missing.json").to_str().unwrap(),
        );
        guard.set("WEB_ACTION_POLICY", legacy.to_str().unwrap());

        let err = ActionPolicy::load_from_env().unwrap_err();
        assert!(err.contains("missing.json"));
    }

    #[test]
    fn test_policy_schema_rejects_unknown_fields_invalid_default_and_empty_actions() {
        let dir = tempfile::tempdir().unwrap();

        let unknown = dir.path().join("unknown.json");
        fs::write(&unknown, r#"{"defualt":"deny"}"#).unwrap();
        assert!(ActionPolicy::load(unknown.to_str().unwrap()).is_err());

        let invalid_default = dir.path().join("invalid-default.json");
        fs::write(&invalid_default, r#"{"default":"denyy"}"#).unwrap();
        assert!(ActionPolicy::load(invalid_default.to_str().unwrap()).is_err());

        let empty_action = dir.path().join("empty-action.json");
        fs::write(&empty_action, r#"{"deny":[" "]}"#).unwrap();
        assert!(ActionPolicy::load(empty_action.to_str().unwrap()).is_err());

        let padded_action = dir.path().join("padded-action.json");
        fs::write(&padded_action, r#"{"deny":["navigate "]}"#).unwrap();
        assert!(ActionPolicy::load(padded_action.to_str().unwrap()).is_err());

        let null_field = dir.path().join("null-field.json");
        fs::write(&null_field, r#"{"deny":null}"#).unwrap();
        assert!(ActionPolicy::load(null_field.to_str().unwrap()).is_err());
    }

    #[test]
    fn test_policy_reload_preserves_last_known_good_policy() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("policy.json");
        fs::write(&path, r#"{"deny":["navigate"]}"#).unwrap();
        let mut policy = ActionPolicy::load(path.to_str().unwrap()).unwrap();

        fs::write(&path, "not json").unwrap();
        assert!(policy.reload().is_err());
        assert!(matches!(policy.check("navigate"), PolicyResult::Deny(_)));

        fs::write(&path, r#"{"deny":["click"]}"#).unwrap();
        policy.reload().unwrap();
        assert_eq!(policy.check("navigate"), PolicyResult::Allow);
        assert!(matches!(policy.check("click"), PolicyResult::Deny(_)));
    }
}
