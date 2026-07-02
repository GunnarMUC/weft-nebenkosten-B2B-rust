# Schema: NK-Check Industrie
# PostgreSQL 17

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ── Mandanten ──
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    config JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

-- ── Dokument-Sets (eine Abrechnung + Anhange) ──
CREATE TABLE IF NOT EXISTS document_sets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    abr_period VARCHAR(50),
    status VARCHAR(50) DEFAULT 'pending'
        CHECK (status IN ('pending','processing','completed','failed')),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_doc_sets_tenant ON document_sets(tenant_id);
CREATE INDEX idx_doc_sets_status ON document_sets(status);

-- ── Chunks (vom Chunker) ──
CREATE TABLE IF NOT EXISTS chunks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    doc_set_id UUID NOT NULL REFERENCES document_sets(id) ON DELETE CASCADE,
    chunk_idx INTEGER NOT NULL,
    title VARCHAR(500),
    pages INT4RANGE,
    content TEXT,
    tables JSONB,
    annotations JSONB,
    confidence FLOAT CHECK (confidence BETWEEN 0 AND 1),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_chunks_doc_set ON chunks(doc_set_id);

-- ── Analyse-Ergebnisse ──
CREATE TABLE IF NOT EXISTS analysis_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chunk_id UUID NOT NULL REFERENCES chunks(id) ON DELETE CASCADE,
    execution_id VARCHAR(255),
    positions JSONB,
    betrkv_categories JSONB,
    findings JSONB,
    severity INTEGER CHECK (severity BETWEEN 1 AND 5),
    routing VARCHAR(50)
        CHECK (routing IN ('auto','review','escalate')),
    reviewer_decision JSONB,
    confidence FLOAT CHECK (confidence BETWEEN 0 AND 1),
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_analysis_chunk ON analysis_results(chunk_id);
CREATE INDEX idx_analysis_severity ON analysis_results(severity);

-- ── Prfberichte ──
CREATE TABLE IF NOT EXISTS reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    doc_set_id UUID NOT NULL REFERENCES document_sets(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    format VARCHAR(10) CHECK (format IN ('pdf','json','md')),
    content BYTEA,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_reports_doc_set ON reports(doc_set_id);
CREATE INDEX idx_reports_tenant ON reports(tenant_id);

-- ── Extension-Tokens (f Operator-UI Authentifizierung) ──
CREATE TABLE IF NOT EXISTS extension_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    token VARCHAR(255) UNIQUE NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    label VARCHAR(255),
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_tokens_tenant ON extension_tokens(tenant_id);
CREATE INDEX idx_tokens_user ON extension_tokens(user_id);

-- ── Audit-Log ──
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id),
    user_id VARCHAR(255),
    action VARCHAR(255) NOT NULL,
    entity_type VARCHAR(100),
    entity_id UUID,
    details JSONB DEFAULT '{}',
    ip_address INET,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_audit_tenant ON audit_log(tenant_id);
CREATE INDEX idx_audit_action ON audit_log(action);
CREATE INDEX idx_audit_created ON audit_log(created_at);

-- ── Default-Tenant f Entwicklung ──
INSERT INTO tenants (id, name, config)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'Default Tenant (Dev)',
    '{"dev_mode": true}'
) ON CONFLICT (id) DO NOTHING;
