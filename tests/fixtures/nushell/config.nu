# Test fixture config for mcp-server-nu integration tests
$env.TEST_CONFIG_LOADED = "yes"

# Custom command for testing
def test_custom_cmd [] {
  "custom command works"
}

# Set table mode for testing
$env.config.table.mode = "light"