from core.kernel.security import (
    NOTEBOOK_SYNC_FS_SCOPE,
    NOTEBOOK_SYNC_NETWORK_SCOPE,
    issue_notebook_sync_token,
    verify_capability_token,
)


def test_issue_notebook_sync_token_scopes() -> None:
    token = issue_notebook_sync_token(client_id="notebook-agent")
    claims = verify_capability_token(token)
    assert NOTEBOOK_SYNC_FS_SCOPE in claims.fs_scopes
    assert NOTEBOOK_SYNC_NETWORK_SCOPE in claims.network_scopes
    assert claims.client_id == "notebook-agent"
