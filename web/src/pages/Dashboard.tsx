import {useEffect, useState} from "react";
import {Button, Card, CardBody, CardHeader, Chip, Dropdown, DropdownItem, DropdownMenu, DropdownTrigger, Input, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader, Progress, Switch, Table, TableBody, TableCell, TableColumn, TableHeader, TableRow, Tooltip, useDisclosure} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {metricsAPI, serverAPI} from "../utils/api";
import {Server, SystemStats} from "../types";
import {motion} from "framer-motion";
import CertificateModal from "../components/CertificateModal.tsx";

export default function Dashboard()
{
    const {isOpen: isCertOpen, onOpen: onCertOpen, onClose: onCertClose} = useDisclosure();
    const [stats, setStats] = useState<SystemStats | null>(null);
    const [servers, setServers] = useState<Server[]>([]);
    const [, setLoading] = useState(true);
    const {isOpen: isCreateOpen, onOpen: onCreateOpen, onClose: onCreateClose} = useDisclosure();
    const [uploadFile, setUploadFile] = useState<File | null>(null);

    // Form state
    const [formData, setFormData] = useState({
        name: "",
        domain: "",
        host: "localhost",
        port: "",
        path: "",
        ssl_enabled: false,
        redirect_to_https: false,
        listen_port: ""
    });

    useEffect(() =>
    {
        loadData().then();
        const interval = setInterval(loadData, 5000); // Refresh every 5 seconds
        return () => clearInterval(interval);
    }, []);

    const loadData = async () =>
    {
        try
        {
            const [statsData, serversData] = await Promise.all([
                metricsAPI.getSystemStats(),
                serverAPI.list()
            ]);
            setStats(statsData);
            setServers(serversData);
        } catch (error)
        {
            console.error("Failed to load data:", error);
        } finally
        {
            setLoading(false);
        }
    };

    const formatBytes = (bytes: number) =>
    {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const sizes = ["B", "KB", "MB", "GB", "TB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return Math.round(bytes / Math.pow(k, i) * 100) / 100 + " " + sizes[i];
    };

    const statusCounts = servers.reduce((acc, server) =>
    {
        acc[server.status] = (acc[server.status] || 0) + 1;
        return acc;
    }, {} as Record<string, number>);

    const handleCreateServer = async () =>
    {
        try
        {
            const data: any = {
                name: formData.name,
                domain: formData.domain,
                host: formData.host,
                port: parseInt(formData.port),
                path: formData.path || undefined,
                ssl_enabled: formData.ssl_enabled,
                redirect_to_https: formData.redirect_to_https,
                listen_port: formData.listen_port ? parseInt(formData.listen_port) : undefined
            };

            const server = await serverAPI.create(data);

            if (uploadFile)
            {
                await serverAPI.uploadBinary(server.id, uploadFile);
            }

            await loadData();
            onCreateClose();
            resetForm();
        } catch (error: any)
        {
            alert("Failed to create server: " + error.message);
        }
    };

    const resetForm = () =>
    {
        setFormData({
            name: "",
            domain: "",
            host: "localhost",
            port: "",
            path: "",
            ssl_enabled: false,
            redirect_to_https: false,
            listen_port: ""
        });
        setUploadFile(null);
    };

    const handleServerAction = async (action: string, server: Server) =>
    {
        try
        {
            switch (action)
            {
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
                    if (confirm(`Delete server "${server.name}"?`))
                    {
                        await serverAPI.delete(server.id);
                    }
                    break;
            }
            await loadData();
        } catch (error: any)
        {
            alert("Action failed: " + error.message);
        }
    };

    const getStatusColor = (status: string) =>
    {
        switch (status)
        {
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
        <>
            <CertificateModal isOpen={isCertOpen} onClose={onCertClose}/>

            <div className="p-6 space-y-6">
                <motion.div
                    initial={{opacity: 0, y: -20}}
                    animate={{opacity: 1, y: 0}}
                    transition={{duration: 0.5}}
                >
                    <h1 className="text-3xl font-bold mb-2">Dashboard</h1>
                    <p className="text-foreground/60">Monitor your minipx proxy servers and system resources</p>
                </motion.div>

                {/* Quick Stats */}
                <motion.div
                    className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4"
                    initial={{opacity: 0}}
                    animate={{opacity: 1}}
                    transition={{duration: 0.5, delay: 0.1}}
                >
                    <Card className="animate-scale-in hover:scale-105 transition-transform bg-blue-500/5">
                        <CardBody className="flex flex-row items-center gap-4">
                            <div className="p-3 rounded-full bg-primary/10">
                                <Icon icon="solar:server-bold" width="24" height="24" className="text-primary"/>
                            </div>
                            <div>
                                <p className="text-sm text-foreground/60">Total Servers</p>
                                <p className="text-2xl font-bold">{servers.length}</p>
                            </div>
                        </CardBody>
                    </Card>

                    <Card className="animate-scale-in hover:scale-105 transition-transform bg-blue-500/5" style={{animationDelay: "0.1s"}}>
                        <CardBody className="flex flex-row items-center gap-4">
                            <div className="p-3 rounded-full bg-success/10">
                                <Icon icon="solar:check-circle-bold" width="24" height="24" className="text-success"/>
                            </div>
                            <div>
                                <p className="text-sm text-foreground/60">Running</p>
                                <p className="text-2xl font-bold">{statusCounts.running || 0}</p>
                            </div>
                        </CardBody>
                    </Card>

                    <Card className="animate-scale-in hover:scale-105 transition-transform bg-blue-500/5" style={{animationDelay: "0.2s"}}>
                        <CardBody className="flex flex-row items-center gap-4">
                            <div className="p-3 rounded-full bg-danger/10">
                                <Icon icon="solar:close-circle-bold" width="24" height="24" className="text-danger"/>
                            </div>
                            <div>
                                <p className="text-sm text-foreground/60">Stopped</p>
                                <p className="text-2xl font-bold">{statusCounts.stopped || 0}</p>
                            </div>
                        </CardBody>
                    </Card>

                    <Card className="animate-scale-in hover:scale-105 transition-transform bg-blue-500/5" style={{animationDelay: "0.3s"}}>
                        <CardBody className="flex flex-row items-center gap-4">
                            <div className="p-3 rounded-full bg-warning/10">
                                <Icon icon="solar:shield-keyhole-bold" width="24" height="24" className="text-warning"/>
                            </div>
                            <div>
                                <p className="text-sm text-foreground/60">SSL Enabled</p>
                                <p className="text-2xl font-bold">{servers.filter(s => s.ssl_enabled).length}</p>
                            </div>
                        </CardBody>
                    </Card>
                </motion.div>

                {/* System Resources */}
                {stats && (
                    <motion.div
                        className="grid grid-cols-1 md:grid-cols-2 gap-6"
                        initial={{opacity: 0, y: 20}}
                        animate={{opacity: 1, y: 0}}
                        transition={{duration: 0.5, delay: 0.4}}
                    >
                        <Card
                            className={"bg-blue-500/5"}
                        >
                            <CardHeader className="flex gap-3">
                                <Icon icon="solar:cpu-bold" width="24" height="24" className="text-primary"/>
                                <div>
                                    <p className="text-lg font-semibold">CPU Usage</p>
                                    <p className="text-sm text-foreground/60">Current processor utilization</p>
                                </div>
                            </CardHeader>
                            <CardBody className="space-y-4">
                                <div className="flex items-center justify-between">
                                    <span className="text-3xl font-bold">{stats.cpu_usage.toFixed(1)}%</span>
                                    <Chip
                                        color={stats.cpu_usage > 80 ? "danger" : stats.cpu_usage > 60 ? "warning" : "success"}
                                        variant="flat"
                                    >
                                        {stats.cpu_usage > 80 ? "High" : stats.cpu_usage > 60 ? "Medium" : "Low"}
                                    </Chip>
                                </div>
                                <Progress
                                    value={stats.cpu_usage}
                                    color={stats.cpu_usage > 80 ? "danger" : stats.cpu_usage > 60 ? "warning" : "success"}
                                    className="animate-pulse"
                                />
                            </CardBody>
                        </Card>

                        <Card
                            className={"bg-blue-500/5"}
                        >
                            <CardHeader className="flex gap-3">
                                <Icon icon="solar:chip-bold" width="24" height="24" className="text-secondary"/>
                                <div>
                                    <p className="text-lg font-semibold">Memory Usage</p>
                                    <p className="text-sm text-foreground/60">RAM utilization</p>
                                </div>
                            </CardHeader>
                            <CardBody className="space-y-4">
                                <div className="flex items-center justify-between">
                                    <div>
                                        <span className="text-3xl font-bold">{stats.memory_usage.toFixed(1)}%</span>
                                        <p className="text-sm text-foreground/60 mt-1">
                                            {formatBytes(stats.memory_used)} / {formatBytes(stats.memory_total)}
                                        </p>
                                    </div>
                                    <Chip
                                        color={stats.memory_usage > 80 ? "danger" : stats.memory_usage > 60 ? "warning" : "success"}
                                        variant="flat"
                                    >
                                        {stats.memory_usage > 80 ? "High" : stats.memory_usage > 60 ? "Medium" : "Low"}
                                    </Chip>
                                </div>
                                <Progress
                                    value={stats.memory_usage}
                                    color={stats.memory_usage > 80 ? "danger" : stats.memory_usage > 60 ? "warning" : "success"}
                                    className="animate-pulse"
                                />
                            </CardBody>
                        </Card>

                        <Card
                            className={"bg-blue-500/5"}
                        >
                            <CardHeader className="flex gap-3">
                                <Icon icon="solar:database-bold" width="24" height="24" className="text-warning"/>
                                <div>
                                    <p className="text-lg font-semibold">Disk Usage</p>
                                    <p className="text-sm text-foreground/60">Storage utilization</p>
                                </div>
                            </CardHeader>
                            <CardBody className="space-y-4">
                                <div className="flex items-center justify-between">
                                    <div>
                                        <span className="text-3xl font-bold">{stats.disk_usage.toFixed(1)}%</span>
                                        <p className="text-sm text-foreground/60 mt-1">
                                            {formatBytes(stats.disk_used)} / {formatBytes(stats.disk_total)}
                                        </p>
                                    </div>
                                    <Chip
                                        color={stats.disk_usage > 80 ? "danger" : stats.disk_usage > 60 ? "warning" : "success"}
                                        variant="flat"
                                    >
                                        {stats.disk_usage > 80 ? "High" : stats.disk_usage > 60 ? "Medium" : "Low"}
                                    </Chip>
                                </div>
                                <Progress
                                    value={stats.disk_usage}
                                    color={stats.disk_usage > 80 ? "danger" : stats.disk_usage > 60 ? "warning" : "success"}
                                    className="animate-pulse"
                                />
                            </CardBody>
                        </Card>

                        <Card
                            className={"bg-blue-500/5"}
                        >
                            <CardHeader className="flex gap-3">
                                <Icon icon="solar:wifi-router-bold" width="24" height="24" className="text-success"/>
                                <div>
                                    <p className="text-lg font-semibold">Network</p>
                                    <p className="text-sm text-foreground/60">Data transfer</p>
                                </div>
                            </CardHeader>
                            <CardBody className="space-y-3">
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center gap-2">
                                        <Icon icon="solar:download-bold" width="18" height="18" className="text-success"/>
                                        <span className="text-sm">Download</span>
                                    </div>
                                    <span className="font-semibold">{formatBytes(stats.network_in)}</span>
                                </div>
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center gap-2">
                                        <Icon icon="solar:upload-bold" width="18" height="18" className="text-primary"/>
                                        <span className="text-sm">Upload</span>
                                    </div>
                                    <span className="font-semibold">{formatBytes(stats.network_out)}</span>
                                </div>
                            </CardBody>
                        </Card>
                    </motion.div>
                )}

                {/* Servers Table */}
                <motion.div
                    initial={{opacity: 0, y: 20}}
                    animate={{opacity: 1, y: 0}}
                    transition={{duration: 0.5, delay: 0.6}}
                >
                    <Card
                        className={"bg-blue-500/5"}
                    >
                        <CardHeader className="flex justify-between items-center">
                            <div className="flex items-center gap-2">
                                <Icon icon="solar:server-bold" width="24" height="24" className="text-primary"/>
                                <p className="text-lg font-semibold">Servers</p>
                            </div>
                            <div className={"flex flex-row gap-2"}>

                                <Button
                                    color="primary"
                                    size="sm"
                                    startContent={<Icon icon="solar:add-circle-bold" width="18" height="18"/>}
                                    onPress={onCreateOpen}
                                    className="hover:scale-105 transition-transform"
                                >
                                    Create Server
                                </Button>
                                <Tooltip content={"Manage SSL certificates for your servers"}>
                                    <Button
                                        size="sm"
                                        onPress={onCertOpen}
                                        className="hover:scale-105 transition-transform"
                                        isIconOnly
                                    >
                                        <Icon icon="solar:shield-keyhole-bold" width="18" height="18"/>
                                    </Button>
                                </Tooltip>
                            </div>
                        </CardHeader>
                        <CardBody>
                            <Table aria-label="Servers table" removeWrapper classNames={{th: "bg-blue-500/10"}}>
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
                                                    <Icon icon="solar:server-minimalistic-bold" width="18" height="18" className="text-foreground/60"/>
                                                    <span className="font-semibold">{server.name}</span>
                                                </div>
                                            </TableCell>
                                            <TableCell>
                                                <div className="flex items-center gap-2">
                                                    <Icon icon="solar:global-bold" width="16" height="16" className="text-foreground/60"/>
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
                                                    <Chip size="sm" variant="flat" color="success" startContent={<Icon icon="solar:shield-check-bold" width="14" height="14"/>}>
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
                                                            onPress={() => handleServerAction("stop", server)}
                                                            title="Stop"
                                                        >
                                                            <Icon icon="solar:pause-bold" width="16" height="16"/>
                                                        </Button>
                                                    ) : (
                                                        <Button
                                                            size="sm"
                                                            isIconOnly
                                                            color="success"
                                                            variant="flat"
                                                            onPress={() => handleServerAction("start", server)}
                                                            title="Start"
                                                        >
                                                            <Icon icon="solar:play-bold" width="16" height="16"/>
                                                        </Button>
                                                    )}
                                                    <Button
                                                        size="sm"
                                                        isIconOnly
                                                        variant="flat"
                                                        onPress={() => handleServerAction("restart", server)}
                                                        title="Restart"
                                                    >
                                                        <Icon icon="solar:restart-bold" width="16" height="16"/>
                                                    </Button>
                                                    <Dropdown>
                                                        <DropdownTrigger>
                                                            <Button size="sm" isIconOnly variant="light">
                                                                <Icon icon="solar:menu-dots-bold" width="20" height="20"/>
                                                            </Button>
                                                        </DropdownTrigger>
                                                        <DropdownMenu>
                                                            <DropdownItem
                                                                key="delete"
                                                                className="text-danger"
                                                                color="danger"
                                                                startContent={<Icon icon="solar:trash-bin-trash-bold" width="18" height="18"/>}
                                                                onClick={() => handleServerAction("delete", server)}
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

                {/* Create Server Modal */}
                <Modal isOpen={isCreateOpen} onClose={onCreateClose} size="2xl" scrollBehavior="inside" backdrop={"blur"} className={"bg-background"}>
                    <ModalContent>
                        <ModalHeader className="flex gap-2 items-center">
                            <Icon icon="solar:server-bold" width="24" height="24" className="text-primary"/>
                            Create New Server
                        </ModalHeader>
                        <ModalBody>
                            <div className="space-y-4">
                                <Input
                                    label="Server Name"
                                    placeholder="My Application"
                                    value={formData.name}
                                    onChange={(e) => setFormData({...formData, name: e.target.value})}
                                    isRequired
                                    startContent={<Icon icon="solar:tag-bold" width="18" height="18"/>}
                                    classNames={{inputWrapper: "dark:bg-white/5 bg-black/5"}}
                                />

                                <Input
                                    label="Domain"
                                    placeholder="app.example.com"
                                    value={formData.domain}
                                    onChange={(e) => setFormData({...formData, domain: e.target.value})}
                                    isRequired
                                    startContent={<Icon icon="solar:global-bold" width="18" height="18"/>}
                                    classNames={{inputWrapper: "dark:bg-white/5 bg-black/5"}}
                                />

                                <div className="grid grid-cols-2 gap-4">
                                    <Input
                                        label="Backend Host"
                                        placeholder="localhost"
                                        value={formData.host}
                                        onChange={(e) => setFormData({...formData, host: e.target.value})}
                                        startContent={<Icon icon="solar:server-2-bold" width="18" height="18"/>}
                                        classNames={{inputWrapper: "dark:bg-white/5 bg-black/5"}}
                                    />

                                    <Input
                                        label="Backend Port"
                                        placeholder="8080"
                                        type="number"
                                        value={formData.port}
                                        onChange={(e) => setFormData({...formData, port: e.target.value})}
                                        isRequired
                                        startContent={<Icon icon="solar:hash-bold" width="18" height="18"/>}
                                        classNames={{inputWrapper: "dark:bg-white/5 bg-black/5"}}
                                    />
                                </div>

                                <Input
                                    label="Path Prefix (Optional)"
                                    placeholder="/api/v1"
                                    value={formData.path}

                                    onChange={(e) => setFormData({...formData, path: e.target.value})}
                                    startContent={<Icon icon="solar:folder-path-bold" width="18" height="18"/>}
                                    classNames={{inputWrapper: "dark:bg-white/5 bg-black/5"}}
                                />

                                <Input
                                    label="Listen Port (Optional)"
                                    placeholder="Custom port (leave empty for default)"
                                    type="number"
                                    value={formData.listen_port}
                                    onChange={(e) => setFormData({...formData, listen_port: e.target.value})}
                                    startContent={<Icon icon="solar:soundwave-bold" width="18" height="18"/>}
                                    classNames={{inputWrapper: "dark:bg-white/5 bg-black/5"}}
                                />

                                <div className="flex gap-4">
                                    <Switch
                                        isSelected={formData.ssl_enabled}
                                        onValueChange={(checked) => setFormData({...formData, ssl_enabled: checked})}
                                    >
                                        Enable SSL/HTTPS
                                    </Switch>

                                    <Switch
                                        isSelected={formData.redirect_to_https}
                                        onValueChange={(checked) => setFormData({...formData, redirect_to_https: checked})}
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
                                        <Icon icon="solar:upload-bold-duotone" width="48" height="48" className="mx-auto text-primary mb-2"/>
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
                            <Button variant="light" onPress={onCreateClose}>
                                Cancel
                            </Button>
                            <Button
                                color="primary"
                                onPress={handleCreateServer}
                                isDisabled={!formData.name || !formData.domain || !formData.port}
                                startContent={<Icon icon="solar:check-circle-bold" width="18" height="18"/>}
                            >
                                Create Server
                            </Button>
                        </ModalFooter>
                    </ModalContent>
                </Modal>
            </div>
            <CertificateModal isOpen={isCertOpen} onClose={onCertClose}/>
        </>
    );
}
