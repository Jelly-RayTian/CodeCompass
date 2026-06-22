-- V8: Scan-generation tracking for safe deletion reconciliation.
--
-- Replaces second-resolution last_seen_at comparison with a monotonic
-- generation counter, eliminating timing ambiguity when scans complete
-- within the same Unix-second.

ALTER TABLE indexed_files ADD COLUMN scan_generation INTEGER NOT NULL DEFAULT 0;
