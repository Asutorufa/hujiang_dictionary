import { useEffect, useState } from "react";
import {
  Button,
  Card,
  TextField,
  Select,
  Dialog,
  Flex,
  Text,
  Box,
} from "@radix-ui/themes";
import { useDisclosure } from "@/components";
import { authorizedRequest } from "../lib/api";
import { useLocation } from "wouter";
import { ROUTE_LOGIN } from "../lib/constants";

export type LlmProvider = {
  name: string;
  base_url: string;
  api_key: string;
  provider: string;
  models: string;
  project_id: string;
  location: string;
};

export type Configuration = {
  key: string;
  value: string;
};

export default function Config() {
  const [, setLocation] = useLocation();
  const [providers, setProviders] = useState<LlmProvider[]>([]);
  const [configurations, setConfigurations] = useState<Configuration[]>([]);
  const [activeTab, setActiveTab] = useState("llm");

  const { isOpen, onOpen, onClose } = useDisclosure();
  const [editingProvider, setEditingProvider] = useState<LlmProvider | null>(
    null,
  );

  const {
    isOpen: isConfigOpen,
    onOpen: onConfigOpen,
    onClose: onConfigClose,
  } = useDisclosure();
  const [editingConfig, setEditingConfig] = useState<Configuration | null>(
    null,
  );

  const fetchProviders = async () => {
    try {
      const res = await authorizedRequest("/llm/list", {
        method: "POST",
      });
      if (!res.ok) {
        if (res.status === 401) {
          setLocation(ROUTE_LOGIN);
        }
        return;
      }
      const data = (await res.json()) as LlmProvider[];
      setProviders(data);
    } catch (e) {
      console.error(e);
    }
  };

  const fetchConfigurations = async () => {
    try {
      const res = await authorizedRequest("/config/list", {
        method: "POST",
      });
      if (!res.ok) {
        if (res.status === 401) {
          setLocation(ROUTE_LOGIN);
        }
        return;
      }
      const data = (await res.json()) as Configuration[];
      setConfigurations(data);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    fetchProviders();
    fetchConfigurations();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleDelete = async (name: string) => {
    if (
      !window.confirm(`Are you sure you want to delete provider "${name}"?`)
    ) {
      return;
    }
    try {
      await authorizedRequest("/llm/delete", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name }),
      });
      fetchProviders();
    } catch (e) {
      console.error(e);
    }
  };

  const handleSave = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const data = Object.fromEntries(
      formData.entries(),
    ) as unknown as LlmProvider;

    try {
      await authorizedRequest("/llm/save", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      onClose();
      fetchProviders();
    } catch (err) {
      console.error(err);
      alert(
        `Failed to save provider: ${err instanceof Error ? err.message : String(err)}`,
      );
    }
  };

  const openAddModal = () => {
    setEditingProvider(null);
    onOpen();
  };

  const handleConfigSave = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const data = Object.fromEntries(
      formData.entries(),
    ) as unknown as Configuration;

    try {
      await authorizedRequest("/config/save", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      onConfigClose();
      fetchConfigurations();
    } catch (err) {
      console.error(err);
      alert(
        `Failed to save configuration: ${err instanceof Error ? err.message : String(err)}`,
      );
    }
  };

  const openEditModal = (provider: LlmProvider) => {
    setEditingProvider(provider);
    onOpen();
  };

  const openConfigAddModal = () => {
    setEditingConfig(null);
    onConfigOpen();
  };

  const openConfigEditModal = (config: Configuration) => {
    setEditingConfig(config);
    onConfigOpen();
  };

  return (
    <div className="container mx-auto p-4 max-w-4xl pb-32">
      <Flex gap="4" mb="6" align="center">
        <Button
          variant={activeTab === "llm" ? "solid" : "soft"}
          onClick={() => setActiveTab("llm")}
        >
          LLM Providers
        </Button>
        <Button
          variant={activeTab === "config" ? "solid" : "soft"}
          onClick={() => setActiveTab("config")}
        >
          General Configurations
        </Button>
      </Flex>

      {activeTab === "llm" && (
        <>
          <Flex justify="between" align="center" mb="6">
            <Text size="6" weight="bold">
              LLM Providers
            </Text>
            <Button color="blue" onClick={openAddModal}>
              Add Provider
            </Button>
          </Flex>

          <div className="grid gap-4 md:grid-cols-2">
            {providers.map((p) => (
              <Card key={p.name} size="2">
                <Flex justify="between" align="start">
                  <Box>
                    <Text size="4" weight="bold" as="div">
                      {p.name}
                    </Text>
                    <Text size="2" color="gray" as="div">
                      {p.provider}
                    </Text>
                    {p.base_url && (
                      <Text size="1" className="truncate" as="div">
                        {p.base_url}
                      </Text>
                    )}
                  </Box>
                  <Flex gap="2">
                    <Button
                      size="1"
                      variant="soft"
                      onClick={() => openEditModal(p)}
                    >
                      Edit
                    </Button>
                    <Button
                      size="1"
                      color="red"
                      variant="soft"
                      onClick={() => handleDelete(p.name)}
                    >
                      Delete
                    </Button>
                  </Flex>
                </Flex>
              </Card>
            ))}
          </div>
        </>
      )}

      {activeTab === "config" && (
        <>
          <Flex justify="between" align="center" mb="6">
            <Text size="6" weight="bold">
              General Configurations
            </Text>
            <Button color="blue" onClick={openConfigAddModal}>
              Add Configuration
            </Button>
          </Flex>

          <div className="grid gap-4 md:grid-cols-2">
            {configurations.map((c) => (
              <Card key={c.key} size="2">
                <Flex justify="between" align="start">
                  <Box>
                    <Text size="4" weight="bold" as="div">
                      {c.key}
                    </Text>
                    <Text size="2" color="gray" as="div" className="truncate">
                      {c.value}
                    </Text>
                  </Box>
                  <Flex gap="2">
                    <Button
                      size="1"
                      variant="soft"
                      onClick={() => openConfigEditModal(c)}
                    >
                      Edit
                    </Button>
                  </Flex>
                </Flex>
              </Card>
            ))}
          </div>
        </>
      )}

      <Dialog.Root open={isOpen} onOpenChange={onClose}>
        <Dialog.Content maxWidth="450px">
          <form onSubmit={handleSave}>
            <Dialog.Title>
              {editingProvider ? "Edit Provider" : "Add Provider"}
            </Dialog.Title>
            <Flex direction="column" gap="3" mt="4">
              <Flex direction="column" gap="1">
                <Text as="label" size="2" weight="bold">
                  Name
                </Text>
                <TextField.Root
                  name="name"
                  defaultValue={editingProvider?.name}
                  readOnly={!!editingProvider}
                  required
                />
              </Flex>
              <Flex direction="column" gap="1">
                <Text as="label" size="2" weight="bold">
                  Provider Type
                </Text>
                <Select.Root
                  name="provider"
                  defaultValue={
                    editingProvider ? editingProvider.provider : "openai"
                  }
                >
                  <Select.Trigger />
                  <Select.Content>
                    <Select.Item value="openai">OpenAI</Select.Item>
                    <Select.Item value="gemini">Gemini</Select.Item>
                    <Select.Item value="vertexai">VertexAI</Select.Item>
                    <Select.Item value="workersai">Workers AI</Select.Item>
                  </Select.Content>
                </Select.Root>
              </Flex>
              <Flex direction="column" gap="1">
                <Text as="label" size="2" weight="bold">
                  Base URL (Optional)
                </Text>
                <TextField.Root
                  name="base_url"
                  defaultValue={editingProvider?.base_url}
                />
              </Flex>
              <Flex direction="column" gap="1">
                <Text as="label" size="2" weight="bold">
                  API Key
                </Text>
                <TextField.Root
                  name="api_key"
                  type="password"
                  defaultValue={editingProvider?.api_key}
                />
              </Flex>
              <Flex direction="column" gap="1">
                <Text as="label" size="2" weight="bold">
                  Models (comma separated)
                </Text>
                <TextField.Root
                  name="models"
                  defaultValue={editingProvider?.models}
                  required
                />
              </Flex>
              <Flex direction="column" gap="1">
                <Text as="label" size="2" weight="bold">
                  Project ID (for VertexAI)
                </Text>
                <TextField.Root
                  name="project_id"
                  defaultValue={editingProvider?.project_id}
                />
              </Flex>
              <Flex direction="column" gap="1">
                <Text as="label" size="2" weight="bold">
                  Location (for VertexAI)
                </Text>
                <TextField.Root
                  name="location"
                  defaultValue={editingProvider?.location}
                />
              </Flex>
            </Flex>
            <Flex gap="3" mt="4" justify="end">
              <Button
                color="gray"
                variant="soft"
                type="button"
                onClick={onClose}
              >
                Cancel
              </Button>
              <Button color="blue" type="submit">
                Save
              </Button>
            </Flex>
          </form>
        </Dialog.Content>
      </Dialog.Root>

      <Dialog.Root open={isConfigOpen} onOpenChange={onConfigClose}>
        <Dialog.Content maxWidth="450px">
          <form onSubmit={handleConfigSave}>
            <Dialog.Title>
              {editingConfig ? "Edit Configuration" : "Add Configuration"}
            </Dialog.Title>
            <Flex direction="column" gap="3" mt="4">
              <Flex direction="column" gap="1">
                <Text as="label" size="2" weight="bold">
                  Key
                </Text>
                <TextField.Root
                  name="key"
                  defaultValue={editingConfig?.key}
                  readOnly={!!editingConfig}
                  required
                />
              </Flex>
              <Flex direction="column" gap="1">
                <Text as="label" size="2" weight="bold">
                  Value
                </Text>
                <TextField.Root
                  name="value"
                  defaultValue={editingConfig?.value}
                  required
                />
              </Flex>
            </Flex>
            <Flex gap="3" mt="4" justify="end">
              <Button
                color="gray"
                variant="soft"
                type="button"
                onClick={onConfigClose}
              >
                Cancel
              </Button>
              <Button color="blue" type="submit">
                Save
              </Button>
            </Flex>
          </form>
        </Dialog.Content>
      </Dialog.Root>
    </div>
  );
}
