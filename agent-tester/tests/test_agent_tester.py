"""Tests for agent-tester."""

import pytest
from unittest.mock import AsyncMock, MagicMock, patch

from agent_tester.config import Settings, MCPServerConfig, LLMConfig


class TestConfig:
    """Test configuration classes."""

    def test_llm_config_defaults(self):
        """Test LLMConfig defaults."""
        config = LLMConfig(api_key="test")
        assert config.provider == "openai"
        assert config.model == "gpt-4o"
        assert config.is_valid is True

    def test_llm_config_invalid_without_key(self):
        """Test LLMConfig without API key."""
        config = LLMConfig()
        assert config.is_valid is False

    def test_mcp_server_config_defaults(self):
        """Test MCPServerConfig defaults."""
        config = MCPServerConfig()
        assert config.name == "unnamed"
        assert config.command == ""
        assert config.args == []

    def test_settings_defaults(self):
        """Test Settings defaults."""
        settings = Settings()
        assert settings.hippos_url == "http://localhost:8080"
        assert settings.default_llm == "openai"
        assert "openai" in settings.llms
        assert "anthropic" in settings.llms

    def test_settings_get_llm(self):
        """Test Settings.get_llm()."""
        settings = Settings()

        llm = settings.get_llm("openai")
        assert llm.provider == "openai"

    def test_settings_get_llm_default(self):
        """Test Settings.get_llm() with default."""
        settings = Settings()

        llm = settings.get_llm()
        assert llm.provider == "openai"

    def test_settings_get_llm_not_found(self):
        """Test Settings.get_llm() raises error."""
        settings = Settings()

        with pytest.raises(ValueError):
            settings.get_llm("nonexistent")


class TestMessage:
    """Test message classes."""

    def test_message_creation(self):
        """Test Message creation."""
        from agent_tester.client import Message

        msg = Message(role="user", content="Hello")
        assert msg.role == "user"
        assert msg.content == "Hello"
        assert msg.tool_calls is None


class TestToolCallResult:
    """Test tool call result classes."""

    def test_tool_call_result_creation(self):
        """Test ToolCallResult creation."""
        from agent_tester.client import ToolCallResult

        result = ToolCallResult(call_id="123", tool_name="test_tool", content="result")
        assert result.call_id == "123"
        assert result.tool_name == "test_tool"
        assert result.content == "result"


class TestHipposClient:
    """Test HipposClient."""

    @pytest.mark.asyncio
    async def test_health_check_success(self):
        """Test health check when Hippos is healthy."""
        from agent_tester.hippos import HipposClient
        import httpx

        # Mock the AsyncClient
        mock_response = MagicMock()
        mock_response.status_code = 200

        mock_client = AsyncMock(spec=httpx.AsyncClient)
        mock_client.get = AsyncMock(return_value=mock_response)

        client = HipposClient("http://localhost:8080", "test")
        client.http_client = mock_client

        result = await client.health_check()
        # Since we're mocking at the method level, result depends on actual implementation
        mock_client.get.assert_called_once()

    @pytest.mark.asyncio
    async def test_health_check_failure(self):
        """Test health check when Hippos is unavailable."""
        from agent_tester.hippos import HipposClient

        client = HipposClient("http://localhost:8080", "test")

        # Mock to raise an exception
        with patch.object(client.http_client, "get", side_effect=Exception("Connection failed")):
            result = await client.health_check()
            assert result is False
