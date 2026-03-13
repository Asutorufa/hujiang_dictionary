import { useEffect, useState } from "react";
import {
  Button,
  Card,
  CardBody,
  Input,
  Select,
  SelectItem,
  Modal,
  ModalContent,
  ModalHeader,
  ModalBody,
  ModalFooter,
  useDisclosure,
} from "@heroui/react";
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

export default function Llm() {
  const [, setLocation] = useLocation();
  const [providers, setProviders] = useState<LlmProvider[]>([]);
  const { isOpen, onOpen, onClose } = useDisclosure();
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

  useEffect(() => {
    fetchProviders();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleDelete = async (name: string) => {
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
    }
  };

  const openAddModal = () => {
    setEditingProvider(null);
    onOpen();
  };

  const openEditModal = (provider: LlmProvider) => {
    setEditingProvider(provider);
    onOpen();
  };

  return (
    <div className="container mx-auto p-4 max-w-4xl pb-32">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">LLM Providers</h1>
        <Button color="primary" onPress={openAddModal}>
          Add Provider
        </Button>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        {providers.map((p) => (
          <Card key={p.name}>
            <CardBody>
              <div className="flex justify-between items-start">
                <div>
                  <h3 className="text-lg font-semibold">{p.name}</h3>
                  <p className="text-sm text-gray-500">{p.provider}</p>
                  {p.base_url && (
                    <p className="text-xs truncate">{p.base_url}</p>
                  )}
                </div>
                <div className="flex gap-2">
                  <Button size="sm" onPress={() => openEditModal(p)}>
                    Edit
                  </Button>
                  <Button
                    size="sm"
                    color="danger"
                    onPress={() => handleDelete(p.name)}
                  >
                    Delete
                  </Button>
                </div>
              </div>
            </CardBody>
          </Card>
        ))}
      </div>

      <Modal isOpen={isOpen} onClose={onClose}>
        <ModalContent>
          <form onSubmit={handleSave}>
            <ModalHeader>
              {editingProvider ? "Edit Provider" : "Add Provider"}
            </ModalHeader>
            <ModalBody>
              <Input
                name="name"
                label="Name"
                defaultValue={editingProvider?.name}
                isReadOnly={!!editingProvider}
                required
              />
              <Select
                name="provider"
                label="Provider Type"
                defaultSelectedKeys={
                  editingProvider ? [editingProvider.provider] : ["openai"]
                }
              >
                <SelectItem key="openai">OpenAI</SelectItem>
                <SelectItem key="gemini">Gemini</SelectItem>
                <SelectItem key="vertexai">VertexAI</SelectItem>
                <SelectItem key="workersai">Workers AI</SelectItem>
              </Select>
              <Input
                name="base_url"
                label="Base URL (Optional)"
                defaultValue={editingProvider?.base_url}
              />
              <Input
                name="api_key"
                label="API Key"
                type="password"
                defaultValue={editingProvider?.api_key}
              />
              <Input
                name="models"
                label="Models (comma separated)"
                defaultValue={editingProvider?.models}
                required
              />
              <Input
                name="project_id"
                label="Project ID (for VertexAI)"
                defaultValue={editingProvider?.project_id}
              />
              <Input
                name="location"
                label="Location (for VertexAI)"
                defaultValue={editingProvider?.location}
              />
            </ModalBody>
            <ModalFooter>
              <Button color="danger" variant="light" onPress={onClose}>
                Cancel
              </Button>
              <Button color="primary" type="submit">
                Save
              </Button>
            </ModalFooter>
          </form>
        </ModalContent>
      </Modal>
    </div>
  );
}
