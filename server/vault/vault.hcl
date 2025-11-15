# Vault Integrated Storage (Raft) single-node config

ui = true
disable_mlock = true

listener "tcp" {
  address       = "127.0.0.1:8200"
  tls_disable   = 1
}

storage "raft" {
  path    = "/home/noa/dev/workspace/noa_ark_os/noa_ark_os/server/vault/data"
  node_id = "vault-node-1"
}

api_addr     = "http://127.0.0.1:8200"
cluster_addr = "https://127.0.0.1:8201"

telemetry {
  disable_hostname = true
}

