import {useCallback, useEffect, useState} from "react";
import {useDropzone} from "react-dropzone";
import {Button, Card, CardBody, Chip, Divider, Input, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader, Select, SelectItem, Spinner, Switch, Textarea} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {runtimeAPI, serverAPI} from "../utils/api";
import {extractZipArchive, getExecutableFiles} from "../utils/zipExtractor";
import {extractVariables, fillTemplate, getTemplatesForRuntime, STARTUP_TEMPLATES, suggestMainFile} from "../utils/startupTemplates";
import type {ArchiveFile, Runtime} from "../types";

interface CreateServerModalProps
{
    isOpen: boolean;
    onClose: () => void;
    onServerCreated: () => void;
}

export default function CreateServerModal({isOpen, onClose, onServerCreated}: CreateServerModalProps)
{
    const [uploadFile, setUploadFile] = useState<File | null>(null);
    const [isExtracting, setIsExtracting] = useState(false);
    const [archiveFiles, setArchiveFiles] = useState<ArchiveFile[]>([]);
    const [runtimes, setRuntimes] = useState<Runtime[]>([]);
    const [isLoadingRuntimes, setIsLoadingRuntimes] = useState(false);
    const [selectedTemplate, setSelectedTemplate] = useState<string>("");
    const [templateVariables, setTemplateVariables] = useState<Record<string, string>>({});

    const [formData, setFormData] = useState({
        name: "",
        domain: "",
        host: "localhost",
        port: "",
        path: "",
        ssl_enabled: false,
        redirect_to_https: false,
        listen_port: "",
        runtime_id: "",
        main_executable: "",
        startup_command: ""
    });

    // Fetch runtimes on mount
    useEffect(() =>
    {
        if (isOpen)
        {
            loadRuntimes();
        }
    }, [isOpen]);

    const loadRuntimes = async () =>
    {
        setIsLoadingRuntimes(true);
        try
        {
            const data = await runtimeAPI.list();
            setRuntimes(data);
        } catch (error: any)
        {
            console.error("Failed to load runtimes:", error);
        } finally
        {
            setIsLoadingRuntimes(false);
        }
    };

    const detectRuntimes = async () =>
    {
        setIsLoadingRuntimes(true);
        try
        {
            const data = await runtimeAPI.detect();
            setRuntimes(data);
        } catch (error: any)
        {
            alert("Failed to detect runtimes: " + error.message);
        } finally
        {
            setIsLoadingRuntimes(false);
        }
    };

    // Handle file drop
    const onDrop = useCallback(async (acceptedFiles: File[]) =>
    {
        const file = acceptedFiles[0];
        if (!file) return;

        setUploadFile(file);

        // If it's a zip file, extract it
        if (file.name.toLowerCase().endsWith(".zip"))
        {
            setIsExtracting(true);
            try
            {
                const files = await extractZipArchive(file);
                setArchiveFiles(files);
                console.log("Extracted files:", files);
            } catch (error)
            {
                console.error("Failed to extract archive:", error);
                alert("Failed to extract archive");
            } finally
            {
                setIsExtracting(false);
            }
        } else
        {
            setArchiveFiles([]);
        }
    }, []);

    const {getRootProps, getInputProps, isDragActive} = useDropzone({
        onDrop,
        multiple: false,
        accept: {
            "application/zip": [".zip"],
            "application/x-7z-compressed": [".7z"],
            "application/x-tar": [".tar"],
            "application/gzip": [".gz", ".tgz"],
            "application/octet-stream": [".exe", ".jar", ".bin"]
        }
    });

    // Handle runtime selection
    const handleRuntimeChange = (runtimeId: string) =>
    {
        setFormData({...formData, runtime_id: runtimeId});

        const runtime = runtimes.find(r => r.id === runtimeId);
        if (runtime && archiveFiles.length > 0)
        {
            // Suggest main file based on runtime type
            const suggested = suggestMainFile(
                archiveFiles.map(f => f.path),
                runtime.runtime_type
            );
            if (suggested)
            {
                setFormData(prev => ({...prev, main_executable: suggested}));
            }
        }

        // Reset template selection when runtime changes
        setSelectedTemplate("");
        setTemplateVariables({});
    };

    // Handle template selection
    const handleTemplateSelect = (templateName: string) =>
    {
        setSelectedTemplate(templateName);
        const template = STARTUP_TEMPLATES.find(t => t.name === templateName);

        if (template)
        {
            const variables = extractVariables(template.template);
            const newVars: Record<string, string> = {};

            // Pre-fill some variables
            variables.forEach(v =>
            {
                if (v === "JAR_FILE" && formData.main_executable)
                {
                    newVars[v] = formData.main_executable;
                } else if (v === "DLL_FILE" && formData.main_executable)
                {
                    newVars[v] = formData.main_executable;
                } else if (v === "MAIN_FILE" && formData.main_executable)
                {
                    newVars[v] = formData.main_executable;
                } else if (v === "BINARY_FILE" && formData.main_executable)
                {
                    newVars[v] = formData.main_executable;
                } else if (v === "MEMORY")
                {
                    newVars[v] = "4";
                } else if (v === "MIN_MEMORY")
                {
                    newVars[v] = "1";
                } else if (v === "ENV")
                {
                    newVars[v] = "production";
                } else
                {
                    newVars[v] = "";
                }
            });

            setTemplateVariables(newVars);

            // Update startup command
            const filled = fillTemplate(template.template, newVars);
            setFormData(prev => ({...prev, startup_command: filled}));
        }
    };

    // Update template variable
    const updateTemplateVariable = (key: string, value: string) =>
    {
        const newVars = {...templateVariables, [key]: value};
        setTemplateVariables(newVars);

        const template = STARTUP_TEMPLATES.find(t => t.name === selectedTemplate);
        if (template)
        {
            const filled = fillTemplate(template.template, newVars);
            setFormData(prev => ({...prev, startup_command: filled}));
        }
    };

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
                listen_port: formData.listen_port ? parseInt(formData.listen_port) : undefined,
                runtime_id: formData.runtime_id || undefined,
                main_executable: formData.main_executable || undefined,
                startup_command: formData.startup_command || undefined
            };

            const server = await serverAPI.create(data);

            if (uploadFile)
            {
                await serverAPI.uploadBinary(server.id, uploadFile);
            }

            onServerCreated();
            onClose();
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
            listen_port: "",
            runtime_id: "",
            main_executable: "",
            startup_command: ""
        });
        setUploadFile(null);
        setArchiveFiles([]);
        setSelectedTemplate("");
        setTemplateVariables({});
    };

    const selectedRuntime = runtimes.find(r => r.id === formData.runtime_id);
    const availableTemplates = selectedRuntime
        ? getTemplatesForRuntime(selectedRuntime.runtime_type)
        : [];
    const executableFiles = getExecutableFiles(archiveFiles);

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            size="4xl"
            scrollBehavior="inside"
            backdrop="blur"
            className="bg-background"
        >
            <ModalContent>
                <ModalHeader className="flex gap-2 items-center">
                    <Icon icon="solar:server-bold" width="24" height="24" className="text-primary"/>
                    Create New Server
                </ModalHeader>
                <ModalBody>
                    <div className="space-y-4">
                        {/* Basic Information */}
                        <div className="space-y-4">
                            <h3 className="text-lg font-semibold">Basic Information</h3>
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
                        </div>

                        <Divider/>

                        {/* Upload Section */}
                        <div className="space-y-4">
                            <h3 className="text-lg font-semibold">Upload Binary or Archive</h3>
                            <div
                                {...getRootProps()}
                                className={`border-2 border-dashed rounded-lg p-8 text-center transition-all cursor-pointer ${
                                    isDragActive
                                        ? "border-primary bg-primary/10"
                                        : "border-default-200 hover:border-primary"
                                }`}
                            >
                                <input {...getInputProps()} />
                                {isExtracting ? (
                                    <div className="flex flex-col items-center gap-2">
                                        <Spinner size="lg" color="primary"/>
                                        <p className="font-semibold">Extracting archive...</p>
                                    </div>
                                ) : (
                                    <>
                                        <Icon icon="solar:upload-bold-duotone" width="48" height="48" className="mx-auto text-primary mb-2"/>
                                        <p className="font-semibold">
                                            {isDragActive ? "Drop the file here" : "Drag & drop or click to select"}
                                        </p>
                                        <p className="text-sm text-foreground/60 mt-1">
                                            Supports .zip, .7z, .tar, .gz, .tgz, .exe, .jar
                                        </p>
                                        {uploadFile && (
                                            <Chip className="mt-3" color="success" variant="flat">
                                                <Icon icon="solar:file-check-bold" width="16" height="16" className="mr-1"/>
                                                {uploadFile.name} ({(uploadFile.size / 1024 / 1024).toFixed(2)} MB)
                                            </Chip>
                                        )}
                                    </>
                                )}
                            </div>

                            {archiveFiles.length > 0 && (
                                <Card className="bg-default-50">
                                    <CardBody>
                                        <p className="text-sm font-semibold mb-2">
                                            <Icon icon="solar:archive-bold" width="16" height="16" className="inline mr-1"/>
                                            Archive Contents: {archiveFiles.length} files
                                            {executableFiles.length > 0 && ` (${executableFiles.length} executable)`}
                                        </p>
                                        <div className="max-h-40 overflow-y-auto text-xs space-y-1">
                                            {executableFiles.slice(0, 10).map((file, idx) => (
                                                <div key={idx} className="flex items-center gap-2">
                                                    <Icon icon="solar:file-code-bold" width="14" height="14" className="text-warning"/>
                                                    <span className="font-mono">{file.path}</span>
                                                </div>
                                            ))}
                                            {executableFiles.length > 10 && (
                                                <p className="text-foreground/60 italic">
                                                    + {executableFiles.length - 10} more files...
                                                </p>
                                            )}
                                        </div>
                                    </CardBody>
                                </Card>
                            )}
                        </div>

                        <Divider/>

                        {/* Runtime Selection */}
                        <div className="space-y-4">
                            <div className="flex items-center justify-between">
                                <h3 className="text-lg font-semibold">Runtime Configuration</h3>
                                <Button
                                    size="sm"
                                    variant="flat"
                                    color="primary"
                                    startContent={<Icon icon="solar:refresh-bold" width="16" height="16"/>}
                                    onPress={detectRuntimes}
                                    isLoading={isLoadingRuntimes}
                                >
                                    Detect Runtimes
                                </Button>
                            </div>

                            <Select
                                label="Runtime"
                                placeholder="Select a runtime (optional)"
                                selectedKeys={formData.runtime_id ? [formData.runtime_id] : []}
                                onChange={(e) => handleRuntimeChange(e.target.value)}
                                startContent={<Icon icon="solar:code-bold" width="18" height="18"/>}
                                classNames={{trigger: "dark:bg-white/5 bg-black/5"}}
                            >
                                {runtimes.map((runtime) => (
                                    <SelectItem key={runtime.id} textValue={runtime.display_name} description={<span className={"text-tiny text-primary-700 italic opacity-50"}>{runtime.executable_path}</span>}>
                                        <div className="flex items-center gap-2">
                                            <Icon
                                                icon={
                                                    runtime.runtime_type === "java" ? "simple-icons:oracle" :
                                                        runtime.runtime_type === "dotnet" ? "simple-icons:dotnet" :
                                                            runtime.runtime_type === "nodejs" ? "simple-icons:nodedotjs" :
                                                                runtime.runtime_type === "python" ? "simple-icons:python" :
                                                                    "solar:code-bold"
                                                }
                                                width="16"
                                                height="16"
                                            />
                                            <p>{runtime.display_name} <span className={"text-tiny opacity-50 italic"}>{runtime.version}</span> </p>
                                        </div>
                                    </SelectItem>
                                ))}
                            </Select>

                            {archiveFiles.length > 0 && formData.runtime_id && (
                                <Select
                                    label="Main Executable"
                                    placeholder="Select the main executable file"
                                    selectedKeys={formData.main_executable ? [formData.main_executable] : []}
                                    onChange={(e) => setFormData({...formData, main_executable: e.target.value})}
                                    startContent={<Icon icon="solar:file-code-bold" width="18" height="18"/>}
                                    classNames={{trigger: "dark:bg-white/5 bg-black/5"}}
                                >
                                    {archiveFiles
                                        .filter(f => f.isExecutable)
                                        .map((file) => (
                                            <SelectItem key={file.path} textValue={file.name}>
                                                {file.path}
                                            </SelectItem>
                                        ))}
                                </Select>
                            )}

                            {availableTemplates.length > 0 && (
                                <Select
                                    label="Startup Template"
                                    placeholder="Select a startup command template"
                                    selectedKeys={selectedTemplate ? [selectedTemplate] : []}
                                    onChange={(e) => handleTemplateSelect(e.target.value)}
                                    startContent={<Icon icon="solar:document-text-bold" width="18" height="18"/>}
                                    classNames={{trigger: "dark:bg-white/5 bg-black/5"}}
                                >
                                    {availableTemplates.map((template) => (
                                        <SelectItem key={template.name} textValue={template.name}>
                                            {template.name}
                                        </SelectItem>
                                    ))}
                                </Select>
                            )}

                            {selectedTemplate && Object.keys(templateVariables).length > 0 && (
                                <Card className="bg-default-50">
                                    <CardBody className="space-y-2">
                                        <p className="text-sm font-semibold mb-2">Template Variables</p>
                                        {Object.entries(templateVariables).map(([key, value]) => (
                                            <Input
                                                key={key}
                                                label={key.replace(/_/g, " ")}
                                                placeholder={`Enter ${key.toLowerCase()}`}
                                                value={value}
                                                onChange={(e) => updateTemplateVariable(key, e.target.value)}
                                                size="sm"
                                                classNames={{inputWrapper: "dark:bg-white/10 bg-black/10"}}
                                            />
                                        ))}
                                    </CardBody>
                                </Card>
                            )}

                            <Textarea
                                label="Startup Command"
                                placeholder="Enter startup command (optional)"
                                value={formData.startup_command}
                                onChange={(e) => setFormData({...formData, startup_command: e.target.value})}
                                minRows={2}
                                classNames={{inputWrapper: "dark:bg-white/5 bg-black/5"}}
                            />
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
                        startContent={<Icon icon="solar:check-circle-bold" width="18" height="18"/>}
                    >
                        Create Server
                    </Button>
                </ModalFooter>
            </ModalContent>
        </Modal>
    );
}
