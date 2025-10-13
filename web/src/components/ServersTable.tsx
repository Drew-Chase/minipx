import { Card, CardBody, CardHeader, Button, Chip, Table, TableHeader, TableColumn, TableBody, TableRow, TableCell, Dropdown, DropdownTrigger, DropdownMenu, DropdownItem, Tooltip } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { motion } from "framer-motion";
import { Server } from "../types";

interface ServersTableProps {
  servers: Server[];
  onCreateOpen: () => void;
  onCertOpen: () => void;
  onServerAction: (action: string, server: Server) => void;
}

export default function ServersTable({ servers, onCreateOpen, onCertOpen, onServerAction }: ServersTableProps) {
  const getStatusColor = (status: string) => {
    switch (status) {
      case "running":
        return "success";
      case "stopped":
        return "default";
      case "error":
        return "danger";
      case "restarting":
        return "warning";
      default:
        return "default";
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5, delay: 0.6 }}
    >
      <Card className="bg-blue-500/5">
        <CardHeader className="flex justify-between items-center">
          <div className="flex items-center gap-2">
            <Icon icon="solar:server-bold" width="24" height="24" className="text-primary" />
            <p className="text-lg font-semibold">Servers</p>
          </div>
          <div className="flex flex-row gap-2">
            <Button
              color="primary"
              size="sm"
              startContent={<Icon icon="solar:add-circle-bold" width="18" height="18" />}
              onPress={onCreateOpen}
              className="hover:scale-105 transition-transform"
            >
              Create Server
            </Button>
            <Tooltip content="Manage SSL certificates for your servers">
              <Button
                size="sm"
                onPress={onCertOpen}
                className="hover:scale-105 transition-transform"
                isIconOnly
              >
                <Icon icon="solar:shield-keyhole-bold" width="18" height="18" />
              </Button>
            </Tooltip>
          </div>
        </CardHeader>
        <CardBody>
          <Table aria-label="Servers table" removeWrapper classNames={{ th: "bg-blue-500/10" }}>
            <TableHeader>
              <TableColumn>NAME</TableColumn>
              <TableColumn>DOMAIN</TableColumn>
              <TableColumn>HOST:PORT</TableColumn>
              <TableColumn>STATUS</TableColumn>
              <TableColumn>SSL</TableColumn>
              <TableColumn>ACTIONS</TableColumn>
            </TableHeader>
            <TableBody emptyContent="No servers configured">
              {servers.map((server) => (
                <TableRow key={server.id}>
                  <TableCell>
                    <div className="flex items-center gap-2">
                      <Icon icon="solar:server-minimalistic-bold" width="18" height="18" className="text-foreground/60" />
                      <span className="font-semibold">{server.name}</span>
                    </div>
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center gap-2">
                      <Icon icon="solar:global-bold" width="16" height="16" className="text-foreground/60" />
                      <span>{server.domain}</span>
                    </div>
                  </TableCell>
                  <TableCell>
                    <code className="text-xs bg-default-100 px-2 py-1 rounded">
                      {server.host}:{server.port}
                    </code>
                  </TableCell>
                  <TableCell>
                    <Chip color={getStatusColor(server.status)} variant="flat" size="sm">
                      {server.status}
                    </Chip>
                  </TableCell>
                  <TableCell>
                    {server.ssl_enabled ? (
                      <Chip size="sm" variant="flat" color="success" startContent={<Icon icon="solar:shield-check-bold" width="14" height="14" />}>
                        Enabled
                      </Chip>
                    ) : (
                      <Chip size="sm" variant="flat" color="default">
                        Disabled
                      </Chip>
                    )}
                  </TableCell>
                  <TableCell>
                    <div className="flex gap-1">
                      {server.status === "running" ? (
                        <Button
                          size="sm"
                          isIconOnly
                          color="danger"
                          variant="flat"
                          onPress={() => onServerAction("stop", server)}
                          title="Stop"
                        >
                          <Icon icon="solar:pause-bold" width="16" height="16" />
                        </Button>
                      ) : (
                        <Button
                          size="sm"
                          isIconOnly
                          color="success"
                          variant="flat"
                          onPress={() => onServerAction("start", server)}
                          title="Start"
                        >
                          <Icon icon="solar:play-bold" width="16" height="16" />
                        </Button>
                      )}
                      <Button
                        size="sm"
                        isIconOnly
                        variant="flat"
                        onPress={() => onServerAction("restart", server)}
                        title="Restart"
                      >
                        <Icon icon="solar:restart-bold" width="16" height="16" />
                      </Button>
                      <Dropdown>
                        <DropdownTrigger>
                          <Button size="sm" isIconOnly variant="light">
                            <Icon icon="solar:menu-dots-bold" width="20" height="20" />
                          </Button>
                        </DropdownTrigger>
                        <DropdownMenu>
                          <DropdownItem
                            key="delete"
                            className="text-danger"
                            color="danger"
                            startContent={<Icon icon="solar:trash-bin-trash-bold" width="18" height="18" />}
                            onClick={() => onServerAction("delete", server)}
                          >
                            Delete
                          </DropdownItem>
                        </DropdownMenu>
                      </Dropdown>
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardBody>
      </Card>
    </motion.div>
  );
}
