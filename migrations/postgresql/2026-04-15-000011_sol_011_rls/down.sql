-- TASK-011-011: Rollback PostgreSQL RLS for Multi-Tenancy

DROP POLICY IF EXISTS tenant_isolation_audit_entries ON audit_entries;
ALTER TABLE audit_entries DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation_organizations ON organizations;
ALTER TABLE organizations DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation_users ON users;
ALTER TABLE users DISABLE ROW LEVEL SECURITY;

DROP FUNCTION IF EXISTS current_tenant();
DROP FUNCTION IF EXISTS set_current_tenant(TEXT);
