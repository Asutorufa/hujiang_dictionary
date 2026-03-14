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
import { addToast, ConfirmModal, useDisclosure } from "@/components";
import { authorizedRequest } from "../lib/api";
import { useLocation } from "wouter";
import { ROUTE_LOGIN } from "../lib/constants";
import { PageContainer } from "@/ui/PageContainer";
import { PageHeader } from "@/ui/PageHeader";

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
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null);

  const { isOpen, onOpenChange } = useDisclosure();
  const [editingProvider, setEditingProvider] = useState<LlmProvider | null>(
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

  const handleDelete = (name: string) => {
    setDeleteTarget(name);
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
      onOpenChange(false);
      fetchProviders();
    } catch (err) {
      console.error(err);
      addToast({
        title: "Failed to save provider",
        description: err instanceof Error ? err.message : String(err),
        color: "danger",
      });
    }
  };

  const openAddModal = () => {
    setEditingProvider(null);
    onOpenChange(true);
  };

  const handleGeneralConfigSave = async (
    e: React.FormEvent<HTMLFormElement>,
  ) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);

    const telegramToken = formData.get("TELEGRAM_TOKEN") as string;
    const allowUsers = formData.get("ALLOW_USERS") as string;
    const maintainerId = formData.get("MAINTAINER_ID") as string;

    const formattedAllowUsers = allowUsers
      .split(",")
      .map((u) => u.trim())
      .filter((u) => u.length > 0)
      .join(",");

    const updates = [
      { key: "TELEGRAM_TOKEN", value: telegramToken.trim() },
      { key: "ALLOW_USERS", value: formattedAllowUsers },
      { key: "MAINTAINER_ID", value: maintainerId.trim() },
    ];

    try {
      await Promise.all(
        updates.map((config) =>
          authorizedRequest("/config/save", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(config),
          }),
        ),
      );
      fetchConfigurations();
      addToast({
        title: "Configurations saved",
        color: "success",
      });
    } catch (err) {
      console.error(err);
      addToast({
        title: "Failed to save configurations",
        description: err instanceof Error ? err.message : String(err),
        color: "danger",
        timeout: 0,
      });
    }
  };

  const openEditModal = (provider: LlmProvider) => {
    setEditingProvider(provider);
    onOpenChange(true);
  };

  return (
    <PageContainer size="4xl" className="space-y-6">
      <ConfirmModal
        title={
          deleteTarget
            ? `Are you sure you want to delete provider "${deleteTarget}"?`
            : "Are you sure?"
        }
        open={!!deleteTarget}
        color="danger"
        confirmLabel="Delete"
        cancelLabel="Cancel"
        onChange={(open) => {
          if (!open) setDeleteTarget(null);
        }}
        onConfirm={async () => {
          if (!deleteTarget) return;
          try {
            const res = await authorizedRequest("/llm/delete", {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({ name: deleteTarget }),
            });
            if (!res.ok) {
              addToast({
                title: "Delete failed",
                description: `(${res.status}) ${await res.text()}`,
                color: "danger",
              });
              return;
            }
            addToast({
              title: "Provider deleted",
              color: "success",
            });
            fetchProviders();
          } catch (e) {
            addToast({
              title: "Delete failed",
              description: e instanceof Error ? e.message : String(e),
              color: "danger",
            });
          } finally {
            setDeleteTarget(null);
          }
        }}
      />

      <PageHeader
        title="Config"
        subtitle="LLM providers and general settings"
        actions={
          <Flex gap="2" align="center">
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
              General
            </Button>
          </Flex>
        }
      />

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
        <Card
          size="3"
          className="mt-4"
          style={{
            backgroundColor: "var(--color-panel-solid)",
            backdropFilter: "none",
          }}
        >
          <form onSubmit={handleGeneralConfigSave}>
            <Flex direction="column" gap="4">
              <Text size="6" weight="bold" mb="2">
                General Configurations
              </Text>

              <Flex direction="column" gap="1">
                <Text as="label" size="2" weight="bold">
                  Telegram Bot Token (TELEGRAM_TOKEN)
                </Text>
                <TextField.Root
                  name="TELEGRAM_TOKEN"
                  type="password"
                  defaultValue={
                    configurations.find((c) => c.key === "TELEGRAM_TOKEN")
                      ?.value || ""
                  }
                  placeholder="123456789:ABCDefghIJKlmnopQRSTuvwxYZ1234567890"
                />
              </Flex>

              <Flex direction="column" gap="1">
                <Text as="label" size="2" weight="bold">
                  Maintainer ID (MAINTAINER_ID)
                </Text>
                <Text size="1" color="gray">
                  The primary admin user ID who receives cron messages and error
                  reports.
                </Text>
                <TextField.Root
                  name="MAINTAINER_ID"
                  defaultValue={
                    configurations.find((c) => c.key === "MAINTAINER_ID")
                      ?.value || ""
                  }
                  placeholder="123456789"
                />
              </Flex>

              <Flex direction="column" gap="1">
                <Text as="label" size="2" weight="bold">
                  Allowed Users (ALLOW_USERS)
                </Text>
                <Text size="1" color="gray">
                  Comma-separated list of allowed Telegram User IDs.
                </Text>
                <TextField.Root
                  name="ALLOW_USERS"
                  defaultValue={
                    configurations.find((c) => c.key === "ALLOW_USERS")
                      ?.value || ""
                  }
                  placeholder="123456789,987654321"
                />
              </Flex>

              <Flex justify="end" mt="4">
                <Button color="blue" type="submit" size="3">
                  Save Configurations
                </Button>
              </Flex>
            </Flex>
          </form>
        </Card>
      )}

      <Dialog.Root open={isOpen} onOpenChange={onOpenChange}>
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
                onClick={() => onOpenChange(false)}
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
    </PageContainer>
  );
}
