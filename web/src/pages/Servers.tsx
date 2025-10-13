import { useEffect, useState } from "react";
import { Card, CardBody, CardHeader, CardFooter, Button, Chip, Dropdown, DropdownTrigger, DropdownMenu, DropdownItem, Modal, ModalContent, ModalHeader, ModalBody, ModalFooter, Input, Switch, useDisclosure } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { serverAPI } from "../utils/api";
import { Server } from "../types";
import { motion } from "framer-motion";

export default function Servers() {
  const [servers, setServers] = useState<Server[]>([]);
  const [,setLoading] = useState(true);
  const [selectedServer, setSelectedServer] = useState<Server | null>(null);
  const { isOpen, onOpen, onClose } = useDisclosure();
  const { isOpen: isCreateOpen, onOpen: onCreateOpen, onClose: onCreateClose } = useDisclosure();

  // Form state
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
  const [uploadFile, setUploadFile] = useState<File | null>(null);

  useEffect(() => {
    loadServers();
  }, []);

  const loadServers = async () => {
    try {
      const data = await serverAPI.list();
      setServers(data);
    } catch (error) {
      console.error("Failed to load servers:", error);
    } finally {
      setLoading(false);
    }
  };

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

      // Upload binary if provided
      if (uploadFile) {
        await serverAPI.uploadBinary(server.id, uploadFile);
      }

      await loadServers();
      onCreateClose();
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

  const handleServerAction = async (action: string, server: Server) => {
    try {
      switch (action) {
        case "start":
          await serverAPI.start(server.id);
          break;
        case "stop":
          await serverAPI.stop(server.id);
          break;
        case "restart":
          await serverAPI.restart(server.id);
          break;
        case "delete":
          if (confirm(`Delete server "${server.name}"?`)) {
            await serverAPI.delete(server.id);
          }
          break;
      }
      await loadServers();
    } catch (error: any) {
      alert("Action failed: " + error.message);
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case "running": return "success";
      case "stopped": return "default";
      case "error": return "danger";
      case "restarting": return "warning";
      default: return "default";
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "running": return "solar:play-circle-bold";
      case "stopped": return "solar:pause-circle-bold";
      case "error": return "solar:close-circle-bold";
      case "restarting": return "solar:restart-bold";
      default: return "solar:question-circle-bold";
    }
  };

  return (
    <div className="p-6 space-y-6">
      <motion.div
        className="flex justify-between items-center"
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
      >
        <div>
          <h1 className="text-3xl font-bold mb-2">Servers</h1>
          <p className="text-foreground/60">Manage your proxy servers and applications</p>
        </div>
        <Button
          color="primary"
          size="lg"
          startContent={<Icon icon="solar:add-circle-bold" width="20" height="20" />}
          onClick={onCreateOpen}
          className="hover:scale-105 transition-transform"
        >
          Create Server
        </Button>
      </motion.div>

      {/* Server Cards Grid */}
      <motion.div
        className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.2 }}
      >
        {servers.map((server, index) => (
          <motion.div
            key={server.id}
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ delay: index * 0.05 }}
          >
            <Card className="hover:scale-105 transition-all duration-200 hover:shadow-lg">
              <CardHeader className="flex justify-between">
                <div className="flex items-center gap-3">
                  <div className={`p-2 rounded-lg ${server.status === 'running' ? 'bg-success/10' : 'bg-default/10'}`}>
                    <Icon icon={getStatusIcon(server.status)} width="24" height="24" className={`text-${getStatusColor(server.status)}`} />
                  </div>
                  <div>
                    <p className="font-semibold text-lg">{server.name}</p>
                    <p className="text-sm text-foreground/60">{server.domain}</p>
                  </div>
                </div>
                <Chip color={getStatusColor(server.status)} variant="flat" size="sm">
                  {server.status}
                </Chip>
              </CardHeader>

              <CardBody className="space-y-2">
                <div className="flex items-center gap-2 text-sm">
                  <Icon icon="solar:link-bold" width="16" height="16" className="text-foreground/60" />
                  <span className="text-foreground/80">{server.host}:{server.port}</span>
                </div>

                {server.path && (
                  <div className="flex items-center gap-2 text-sm">
                    <Icon icon="solar:folder-path-bold" width="16" height="16" className="text-foreground/60" />
                    <span className="text-foreground/80">{server.path}</span>
                  </div>
                )}

                <div className="flex gap-2 flex-wrap mt-2">
                  {server.ssl_enabled && (
                    <Chip size="sm" variant="flat" color="success" startContent={<Icon icon="solar:shield-check-bold" width="14" height="14" />}>
                      SSL
                    </Chip>
                  )}
                  {server.redirect_to_https && (
                    <Chip size="sm" variant="flat" color="primary" startContent={<Icon icon="solar:restart-bold" width="14" height="14" />}>
                      HTTPS Redirect
                    </Chip>
                  )}
                  {server.listen_port && (
                    <Chip size="sm" variant="flat" startContent={<Icon icon="solar:soundwave-bold" width="14" height="14" />}>
                      Port {server.listen_port}
                    </Chip>
                  )}
                </div>
              </CardBody>

              <CardFooter className="gap-2">
                {server.status === 'running' ? (
                  <Button
                    size="sm"
                    color="danger"
                    variant="flat"
                    onClick={() => handleServerAction('stop', server)}
                    startContent={<Icon icon="solar:pause-bold" width="16" height="16" />}
                  >
                    Stop
                  </Button>
                ) : (
                  <Button
                    size="sm"
                    color="success"
                    variant="flat"
                    onClick={() => handleServerAction('start', server)}
                    startContent={<Icon icon="solar:play-bold" width="16" height="16" />}
                  >
                    Start
                  </Button>
                )}

                <Button
                  size="sm"
                  variant="flat"
                  onClick={() => handleServerAction('restart', server)}
                  startContent={<Icon icon="solar:restart-bold" width="16" height="16" />}
                >
                  Restart
                </Button>

                <Dropdown>
                  <DropdownTrigger>
                    <Button size="sm" isIconOnly variant="light">
                      <Icon icon="solar:menu-dots-bold" width="20" height="20" />
                    </Button>
                  </DropdownTrigger>
                  <DropdownMenu>
                    <DropdownItem
                      key="view"
                      startContent={<Icon icon="solar:eye-bold" width="18" height="18" />}
                      onClick={() => {
                        setSelectedServer(server);
                        onOpen();
                      }}
                    >
                      View Details
                    </DropdownItem>
                    <DropdownItem
                      key="delete"
                      className="text-danger"
                      color="danger"
                      startContent={<Icon icon="solar:trash-bin-trash-bold" width="18" height="18" />}
                      onClick={() => handleServerAction('delete', server)}
                    >
                      Delete
                    </DropdownItem>
                  </DropdownMenu>
                </Dropdown>
              </CardFooter>
            </Card>
          </motion.div>
        ))}
      </motion.div>

      {/* Create Server Modal */}
      <Modal isOpen={isCreateOpen} onClose={onCreateClose} size="2xl" scrollBehavior="inside">
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
              />

              <Input
                label="Domain"
                placeholder="app.example.com"
                value={formData.domain}
                onChange={(e) => setFormData({ ...formData, domain: e.target.value })}
                isRequired
                startContent={<Icon icon="solar:global-bold" width="18" height="18" />}
              />

              <div className="grid grid-cols-2 gap-4">
                <Input
                  label="Backend Host"
                  placeholder="localhost"
                  value={formData.host}
                  onChange={(e) => setFormData({ ...formData, host: e.target.value })}
                  startContent={<Icon icon="solar:server-2-bold" width="18" height="18" />}
                />

                <Input
                  label="Backend Port"
                  placeholder="8080"
                  type="number"
                  value={formData.port}
                  onChange={(e) => setFormData({ ...formData, port: e.target.value })}
                  isRequired
                  startContent={<Icon icon="solar:hash-bold" width="18" height="18" />}
                />
              </div>

              <Input
                label="Path Prefix (Optional)"
                placeholder="/api/v1"
                value={formData.path}
                onChange={(e) => setFormData({ ...formData, path: e.target.value })}
                startContent={<Icon icon="solar:folder-path-bold" width="18" height="18" />}
              />

              <Input
                label="Listen Port (Optional)"
                placeholder="Custom port (leave empty for default)"
                type="number"
                value={formData.listen_port}
                onChange={(e) => setFormData({ ...formData, listen_port: e.target.value })}
                startContent={<Icon icon="solar:soundwave-bold" width="18" height="18" />}
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
                  accept=".zip,.tar,.gz,.tgz,application/*"
                />
                <label htmlFor="file-upload" className="cursor-pointer">
                  <Icon icon="solar:upload-bold-duotone" width="48" height="48" className="mx-auto text-primary mb-2" />
                  <p className="font-semibold">Upload Binary or Archive</p>
                  <p className="text-sm text-foreground/60">Click to browse or drag and drop</p>
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
            <Button variant="light" onPress={onCreateClose}>
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

      {/* Server Details Modal */}
      {selectedServer && (
        <Modal isOpen={isOpen} onClose={onClose} size="xl">
          <ModalContent>
            <ModalHeader>Server Details</ModalHeader>
            <ModalBody>
              <div className="space-y-3">
                <div>
                  <p className="text-sm text-foreground/60">Name</p>
                  <p className="font-semibold">{selectedServer.name}</p>
                </div>
                <div>
                  <p className="text-sm text-foreground/60">Domain</p>
                  <p className="font-semibold">{selectedServer.domain}</p>
                </div>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <p className="text-sm text-foreground/60">Host</p>
                    <p className="font-semibold">{selectedServer.host}</p>
                  </div>
                  <div>
                    <p className="text-sm text-foreground/60">Port</p>
                    <p className="font-semibold">{selectedServer.port}</p>
                  </div>
                </div>
                <div>
                  <p className="text-sm text-foreground/60">Status</p>
                  <Chip color={getStatusColor(selectedServer.status)} className="mt-1">
                    {selectedServer.status}
                  </Chip>
                </div>
              </div>
            </ModalBody>
            <ModalFooter>
              <Button onPress={onClose}>Close</Button>
            </ModalFooter>
          </ModalContent>
        </Modal>
      )}
    </div>
  );
}
