//! All authorization is enforced in the launcher, never entrusted to the model.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::VecDeque;
use tfl_mcp::catalog::Permission;
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
#[derive(Clone, Serialize)]
pub struct Client {
    pub id: String,
    pub name: String,
    pub connected: bool,
    pub approved: bool,
    pub permissions: Vec<String>,
    pub instances: Vec<String>,
    pub expires: u64,
    pub last_activity: u64,
}
#[derive(Clone, Serialize)]
pub struct Plan {
    pub id: String,
    pub client_id: String,
    pub params: Value,
    pub fingerprint: String,
    pub expires: u64,
    pub approved: bool,
    pub decided: bool,
}
#[derive(Clone, Serialize)]
pub struct Job {
    pub id: String,
    pub client_id: String,
    pub tool: String,
    pub instance_id: Option<String>,
    pub status: String,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub created: u64,
    #[serde(skip)]
    pub idempotency: Option<String>,
    #[serde(skip)]
    pub params: Value,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Audit {
    pub at: u64,
    pub client_id: String,
    pub action: String,
    pub decision: String,
}
#[derive(Default)]
pub struct Policy {
    pub enabled: bool,
    pub clients: Vec<Client>,
    pub plans: Vec<Plan>,
    pub jobs: Vec<Job>,
    pub audit: VecDeque<Audit>,
}
impl Policy {
    pub fn record(&mut self, c: &str, a: &str, d: &str) {
        self.audit.push_back(Audit {
            at: now(),
            client_id: c.into(),
            action: a.into(),
            decision: d.into(),
        });
        while self.audit.len() > 300 {
            self.audit.pop_front();
        }
    }
    pub fn connect(&mut self, name: String) -> Result<String, String> {
        if !self.enabled {
            return Err("MCP_DISABLED".into());
        }
        self.clients.retain(|c| c.connected && c.expires > now());
        if self.clients.len() >= 8 {
            return Err("TOO_MANY_CLIENTS".into());
        }
        let id = uuid::Uuid::new_v4().to_string();
        self.clients.push(Client {
            id: id.clone(),
            name,
            connected: true,
            approved: false,
            permissions: vec![],
            instances: vec![],
            expires: now() + 3600,
            last_activity: now(),
        });
        self.record(&id, "connect", "pending_local_approval");
        Ok(id)
    }
    pub fn authorize(
        &mut self,
        id: &str,
        p: Permission,
        instance: Option<&str>,
    ) -> Result<(), String> {
        if !self.enabled {
            return Err("MCP_DISABLED".into());
        }
        let c = self
            .clients
            .iter_mut()
            .find(|c| c.id == id && c.connected)
            .ok_or("SESSION_CLOSED")?;
        if c.expires <= now() {
            return Err("SESSION_EXPIRED; reconnect and approve again".into());
        }
        if !c.approved {
            return Err(
                "APPROVAL_REQUIRED: approve this client in TFL Client → Settings → Integrations"
                    .into(),
            );
        }
        if !c.permissions.iter().any(|s| s == p.name()) {
            return Err(format!("PERMISSION_DENIED: {}", p.name()));
        }
        if instance.is_some_and(|i| {
            !c.instances.is_empty() && !c.instances.iter().any(|allowed| allowed == i)
        }) {
            return Err("INSTANCE_SCOPE_DENIED".into());
        }
        c.last_activity = now();
        Ok(())
    }
    pub fn grant(
        &mut self,
        id: &str,
        permissions: Vec<String>,
        instances: Vec<String>,
    ) -> Result<(), String> {
        if permissions
            .iter()
            .any(|p| !["read", "configure", "operate", "critical"].contains(&p.as_str()))
            || instances.len() > 200
        {
            return Err("INVALID_GRANT".into());
        }
        let c = self
            .clients
            .iter_mut()
            .find(|c| c.id == id && c.connected && c.expires > now())
            .ok_or("SESSION_CLOSED")?;
        c.permissions = permissions;
        c.instances = instances;
        c.approved = true;
        self.record(id, "grant", "approved_locally");
        Ok(())
    }
    pub fn revoke(&mut self, id: &str) {
        if let Some(c) = self.clients.iter_mut().find(|c| c.id == id) {
            c.approved = false;
            c.connected = false;
        }
        self.plans.retain(|p| p.client_id != id);
        self.record(id, "revoke", "revoked");
    }
    pub fn disconnect(&mut self, id: &str) {
        self.revoke(id);
        self.record(id, "disconnect", "closed");
    }
    pub fn plan(
        &mut self,
        client_id: &str,
        params: Value,
        fingerprint: String,
    ) -> Result<Plan, String> {
        self.authorize(
            client_id,
            Permission::Critical,
            params["instance_id"].as_str(),
        )?;
        self.plans.retain(|p| p.expires > now());
        if self.plans.len() >= 50 {
            return Err("TOO_MANY_APPROVALS".into());
        }
        let p = Plan {
            id: uuid::Uuid::new_v4().to_string(),
            client_id: client_id.into(),
            params,
            fingerprint,
            expires: now() + 120,
            approved: false,
            decided: false,
        };
        self.plans.push(p.clone());
        self.record(client_id, "plan", "pending_local_approval");
        Ok(p)
    }
    pub fn decide(&mut self, id: &str, approve: bool) -> Result<(), String> {
        let p = self
            .plans
            .iter_mut()
            .find(|p| p.id == id && !p.decided && p.expires > now())
            .ok_or("APPROVAL_EXPIRED_OR_DECIDED")?;
        p.decided = true;
        p.approved = approve;
        let client = p.client_id.clone();
        self.record(
            &client,
            "approval",
            if approve {
                "approved_locally"
            } else {
                "denied_locally"
            },
        );
        Ok(())
    }
    pub fn consume(&mut self, client: &str, id: &str, fingerprint: &str) -> Result<Plan, String> {
        let i = self
            .plans
            .iter()
            .position(|p| p.id == id && p.client_id == client)
            .ok_or("PLAN_NOT_FOUND")?;
        let p = self.plans[i].clone();
        self.authorize(
            client,
            Permission::Critical,
            p.params["instance_id"].as_str(),
        )?;
        if !p.approved || !p.decided {
            return Err("LOCAL_APPROVAL_REQUIRED".into());
        }
        self.plans.remove(i);
        if p.expires <= now() {
            return Err("APPROVAL_EXPIRED".into());
        }
        if p.fingerprint != fingerprint {
            return Err("STATE_CHANGED; request a new plan".into());
        }
        Ok(p)
    }
    pub fn own_job(&self, c: &str, id: &str) -> Result<Job, String> {
        self.jobs
            .iter()
            .find(|j| j.client_id == c && j.id == id)
            .cloned()
            .ok_or("JOB_NOT_FOUND".into())
    }
    pub fn scope(&self, c: &str, id: &str) -> bool {
        self.clients
            .iter()
            .find(|x| x.id == c)
            .is_some_and(|x| x.instances.is_empty() || x.instances.iter().any(|i| i == id))
    }
    pub fn snapshot(&self) -> Value {
        json!({"enabled":self.enabled,"clients":self.clients,"plans":self.plans,"jobs":self.jobs,"audit":self.audit})
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn approved() -> (Policy, String) {
        let mut p = Policy {
            enabled: true,
            ..Default::default()
        };
        let c = p.connect("test".into()).unwrap();
        p.grant(
            &c,
            vec!["read".into(), "critical".into()],
            vec!["exact-uuid".into()],
        )
        .unwrap();
        (p, c)
    }
    #[test]
    fn disabled_and_unapproved_are_denied() {
        let mut p = Policy::default();
        assert!(p.connect("x".into()).is_err());
        p.enabled = true;
        let c = p.connect("x".into()).unwrap();
        assert!(p.authorize(&c, Permission::Read, None).is_err());
    }
    #[test]
    fn read_cannot_operate_and_scope_is_enforced() {
        let (mut p, c) = approved();
        assert!(p.authorize(&c, Permission::Operate, None).is_err());
        assert!(p.authorize(&c, Permission::Read, Some("other")).is_err());
        assert!(
            p.authorize(&c, Permission::Read, Some("exact-uuid"))
                .is_ok()
        );
    }
    #[test]
    fn approval_is_bound_once_and_revocation_invalidates() {
        let (mut p, c) = approved();
        let a = p
            .plan(
                &c,
                json!({"instance_id":"exact-uuid","action":"delete_instance"}),
                "v1".into(),
            )
            .unwrap();
        assert!(p.consume(&c, &a.id, "v1").is_err());
        p.decide(&a.id, true).unwrap();
        assert!(p.consume(&c, &a.id, "v1").is_ok());
        assert!(p.consume(&c, &a.id, "v1").is_err());
        let a = p
            .plan(&c, json!({"instance_id":"exact-uuid"}), "v1".into())
            .unwrap();
        p.decide(&a.id, true).unwrap();
        p.revoke(&c);
        assert!(p.consume(&c, &a.id, "v1").is_err());
    }
    #[test]
    fn denial_expiry_replay_and_changed_state() {
        let (mut p, c) = approved();
        let a = p
            .plan(&c, json!({"instance_id":"exact-uuid"}), "v1".into())
            .unwrap();
        p.decide(&a.id, false).unwrap();
        assert!(p.consume(&c, &a.id, "v1").is_err());
        assert!(p.decide(&a.id, true).is_err());
        let b = p
            .plan(&c, json!({"instance_id":"exact-uuid"}), "v1".into())
            .unwrap();
        p.decide(&b.id, true).unwrap();
        assert!(p.consume(&c, &b.id, "changed").is_err());
        assert!(p.consume(&c, &b.id, "v1").is_err());
        p.clients[0].expires = 0;
        assert!(p.authorize(&c, Permission::Read, None).is_err());
    }
}
