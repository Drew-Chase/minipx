import { useState } from "react";
import { Modal, ModalContent, ModalHeader, ModalBody, ModalFooter, Button, Input, Switch, Chip } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { serverAPI } from "../utils/api";

interface CreateServerModalProps {
  isOpen: boolean;
  onClose: () => void;
  onServerCreated: () => void;
}

export default function CreateServerModal({ isOpen, onClose, onServerCreated }: CreateServerModalProps) {
  const [uploadFile, setUploadFile] = useState<File | null>(null);
  const [formData, setFormData] = useState({
    name: "",
    domain: "",
    host: "localhost",
    port: "",
    path: "",
    ssl_enabled: false,
    redirect_to_https: false,
    listen_port: "",
  });

  const handleCreateServer = async () => {
    try {
      const data: any = {
        name: formData.name,
        domain: formData.domain,
        host: formData.host,
        port: parseInt(formData.port),
        path: formData.path || undefined,
        ssl_enabled: formData.ssl_enabled,
        redirect_to_https: formData.redirect_to_https,
        listen_port: formData.listen_port ? parseInt(formData.listen_port) : undefined,
      };

      const server = await serverAPI.create(data);

      if (uploadFile) {
        await serverAPI.uploadBinary(server.id, uploadFile);
      }

      onServerCreated();
      onClose();
      resetForm();
    } catch (error: any) {
      alert("Failed to create server: " + error.message);
    }
  };

  const resetForm = () => {
    setFormData({
      name: "",
      domain: "",
      host: "localhost",
      port: "",
      path: "",
      ssl_enabled: false,
      redirect_to_https: false,
      listen_port: "",
    });
    setUploadFile(null);
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} size="2xl" scrollBehavior="inside" backdrop="blur" className="bg-background">
      <ModalContent>
        <ModalHeader className="flex gap-2 items-center">
          <Icon icon="solar:server-bold" width="24" height="24" className="text-primary" />
          Create New Server
        </ModalHeader>
        <ModalBody>
          <div className="space-y-4">
            <Input
              label="Server Name"
              placeholder="My Application"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              isRequired
              startContent={<Icon icon="solar:tag-bold" width="18" height="18" />}
              classNames={{ inputWrapper: "dark:bg-white/5 bg-black/5" }}
            />

            <Input
              label="Domain"
              placeholder="app.example.com"
              value={formData.domain}
              onChange={(e) => setFormData({ ...formData, domain: e.target.value })}
              isRequired
              startContent={<Icon icon="solar:global-bold" width="18" height="18" />}
              classNames={{ inputWrapper: "dark:bg-white/5 bg-black/5" }}
            />

            <div className="grid grid-cols-2 gap-4">
              <Input
                label="Backend Host"
                placeholder="localhost"
                value={formData.host}
                onChange={(e) => setFormData({ ...formData, host: e.target.value })}
                startContent={<Icon icon="solar:server-2-bold" width="18" height="18" />}
                classNames={{ inputWrapper: "dark:bg-white/5 bg-black/5" }}
              />

              <Input
                label="Backend Port"
                placeholder="8080"
                type="number"
                value={formData.port}
                onChange={(e) => setFormData({ ...formData, port: e.target.value })}
                isRequired
                startContent={<Icon icon="solar:hash-bold" width="18" height="18" />}
                classNames={{ inputWrapper: "dark:bg-white/5 bg-black/5" }}
              />
            </div>

            <Input
              label="Path Prefix (Optional)"
              placeholder="/api/v1"
              value={formData.path}
              onChange={(e) => setFormData({ ...formData, path: e.target.value })}
              startContent={<Icon icon="solar:folder-path-bold" width="18" height="18" />}
              classNames={{ inputWrapper: "dark:bg-white/5 bg-black/5" }}
            />

            <Input
              label="Listen Port (Optional)"
              placeholder="Custom port (leave empty for default)"
              type="number"
              value={formData.listen_port}
              onChange={(e) => setFormData({ ...formData, listen_port: e.target.value })}
              startContent={<Icon icon="solar:soundwave-bold" width="18" height="18" />}
              classNames={{ inputWrapper: "dark:bg-white/5 bg-black/5" }}
            />

            <div className="flex gap-4">
              <Switch
                isSelected={formData.ssl_enabled}
                onValueChange={(checked) => setFormData({ ...formData, ssl_enabled: checked })}
              >
                Enable SSL/HTTPS
              </Switch>

              <Switch
                isSelected={formData.redirect_to_https}
                onValueChange={(checked) => setFormData({ ...formData, redirect_to_https: checked })}
              >
                Redirect HTTP to HTTPS
              </Switch>
            </div>

            <div className="border-2 border-dashed border-default-200 rounded-lg p-6 text-center hover:border-primary transition-colors">
              <input
                type="file"
                id="file-upload"
                className="hidden"
                onChange={(e) => setUploadFile(e.target.files?.[0] || null)}
                accept=".zip,.7z,.tar,.gz,.tgz,application/*"
              />
              <label htmlFor="file-upload" className="cursor-pointer">
                <Icon icon="solar:upload-bold-duotone" width="48" height="48" className="mx-auto text-primary mb-2" />
                <p className="font-semibold">Upload Binary or Archive</p>
                <p className="text-sm text-foreground/60">Supports .zip, .7z, .tar, .gz, .tgz</p>
                {uploadFile && (
                  <Chip className="mt-2" color="success" variant="flat">
                    {uploadFile.name}
                  </Chip>
                )}
              </label>
            </div>
          </div>
        </ModalBody>
        <ModalFooter>
          <Button variant="light" onPress={onClose}>
            Cancel
          </Button>
          <Button
            color="primary"
            onPress={handleCreateServer}
            isDisabled={!formData.name || !formData.domain || !formData.port}
            startContent={<Icon icon="solar:check-circle-bold" width="18" height="18" />}
          >
            Create Server
          </Button>
        </ModalFooter>
      </ModalContent>
    </Modal>
  );
}
