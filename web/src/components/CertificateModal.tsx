import {useEffect, useState} from "react";
import {Button, Chip, Dropdown, DropdownItem, DropdownMenu, DropdownTrigger, Input, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader, Switch, Table, TableBody, TableCell, TableColumn, TableHeader, TableRow, useDisclosure} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {certificateAPI} from "../utils/api";
import {Certificate} from "../types";

interface CertificateModalProps
{
    isOpen: boolean;
    onClose: () => void;
}

export default function CertificateModal({isOpen, onClose}: CertificateModalProps)
{
    const [certificates, setCertificates] = useState<Certificate[]>([]);
    const {isOpen: isCreateOpen, onOpen: onCreateOpen, onClose: onCreateClose} = useDisclosure();
    const [formData, setFormData] = useState({
        name: "",
        domain: "",
        is_letsencrypt: true
    });
    const [certFile, setCertFile] = useState<File | null>(null);
    const [keyFile, setKeyFile] = useState<File | null>(null);

    useEffect(() =>
    {
        if (isOpen)
        {
            loadCertificates();
        }
    }, [isOpen]);

    const loadCertificates = async () =>
    {
        try
        {
            const data = await certificateAPI.list();
            setCertificates(data);
        } catch (error)
        {
            console.error("Failed to load certificates:", error);
        }
    };

    const handleCreateCertificate = async () =>
    {
        try
        {
            const cert = await certificateAPI.create(formData);

            if (!formData.is_letsencrypt && certFile)
            {
                await certificateAPI.uploadCertificate(cert.id, certFile, keyFile || undefined);
            }

            await loadCertificates();
            onCreateClose();
            resetForm();
        } catch (error: any)
        {
            alert("Failed to create certificate: " + error.message);
        }
    };

    const handleDeleteCertificate = async (id: string, name: string) =>
    {
        if (confirm(`Delete certificate "${name}"?`))
        {
            try
            {
                await certificateAPI.delete(id);
                await loadCertificates();
            } catch (error: any)
            {
                alert("Failed to delete certificate: " + error.message);
            }
        }
    };

    const resetForm = () =>
    {
        setFormData({
            name: "",
            domain: "",
            is_letsencrypt: true
        });
        setCertFile(null);
        setKeyFile(null);
    };

    return (
        <>
            <Modal isOpen={isOpen} onClose={onClose} size="3xl" scrollBehavior="inside" backdrop={"blur"} className="max-h-[80vh] overflow-y-auto bg-background">
                <ModalContent>
                    <ModalHeader className="flex gap-2 items-center">
                        <Icon icon="solar:shield-keyhole-bold" width="24" height="24" className="text-warning"/>
                        SSL Certificates
                    </ModalHeader>
                    <ModalBody>
                        <div className="space-y-4">
                            <div className="flex justify-between items-center mb-4">
                                <p className="text-sm text-foreground/60">Manage your SSL/TLS certificates for secure connections</p>
                                <Button
                                    color="primary"
                                    size="sm"
                                    startContent={<Icon icon="solar:add-circle-bold" width="18" height="18"/>}
                                    onPress={onCreateOpen}
                                >
                                    Add Certificate
                                </Button>
                            </div>

                            <Table aria-label="Certificates table" removeWrapper classNames={{th: "bg-blue-500/10"}}>
                                <TableHeader>
                                    <TableColumn>NAME</TableColumn>
                                    <TableColumn>DOMAIN</TableColumn>
                                    <TableColumn>TYPE</TableColumn>
                                    <TableColumn>EXPIRY</TableColumn>
                                    <TableColumn>ACTIONS</TableColumn>
                                </TableHeader>
                                <TableBody emptyContent="No certificates configured">
                                    {certificates.map((cert) => (
                                        <TableRow key={cert.id}>
                                            <TableCell>
                                                <div className="flex items-center gap-2">
                                                    <Icon icon="solar:shield-check-bold" width="18" height="18" className="text-success"/>
                                                    <span className="font-semibold">{cert.name}</span>
                                                </div>
                                            </TableCell>
                                            <TableCell>
                                                <code className="text-xs bg-default-100 px-2 py-1 rounded">
                                                    {cert.domain}
                                                </code>
                                            </TableCell>
                                            <TableCell>
                                                {cert.is_letsencrypt ? (
                                                    <Chip size="sm" variant="flat" color="success" startContent={<Icon icon="solar:verified-check-bold" width="14" height="14"/>}>
                                                        Let's Encrypt
                                                    </Chip>
                                                ) : (
                                                    <Chip size="sm" variant="flat" color="primary" startContent={<Icon icon="solar:document-add-bold" width="14" height="14"/>}>
                                                        Custom
                                                    </Chip>
                                                )}
                                            </TableCell>
                                            <TableCell>
                                                {cert.expiry_date ? (
                                                    <span className="text-sm">{new Date(cert.expiry_date).toLocaleDateString()}</span>
                                                ) : (
                                                    <span className="text-sm text-foreground/40">N/A</span>
                                                )}
                                            </TableCell>
                                            <TableCell>
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
                                                            onClick={() => handleDeleteCertificate(cert.id, cert.name)}
                                                        >
                                                            Delete
                                                        </DropdownItem>
                                                    </DropdownMenu>
                                                </Dropdown>
                                            </TableCell>
                                        </TableRow>
                                    ))}
                                </TableBody>
                            </Table>
                        </div>
                    </ModalBody>
                    <ModalFooter>
                        <Button onPress={onClose}>Close</Button>
                    </ModalFooter>
                </ModalContent>
            </Modal>

            {/* Create Certificate Modal */}
            <Modal isOpen={isCreateOpen} onClose={onCreateClose} size="xl" backdrop={"blur"} className={"bg-background"}>
                <ModalContent>
                    <ModalHeader className="flex gap-2 items-center">
                        <Icon icon="solar:shield-plus-bold" width="24" height="24" className="text-primary"/>
                        Add SSL Certificate
                    </ModalHeader>
                    <ModalBody>
                        <div className="space-y-4">
                            <Input
                                label="Certificate Name"
                                placeholder="My SSL Certificate"
                                value={formData.name}
                                onChange={(e) => setFormData({...formData, name: e.target.value})}
                                isRequired
                                startContent={<Icon icon="solar:tag-bold" width="18" height="18"/>}
                                classNames={{inputWrapper: "dark:bg-white/5 bg-black/5"}}
                            />

                            <Input
                                label="Domain"
                                placeholder="example.com or *.example.com"
                                value={formData.domain}
                                onChange={(e) => setFormData({...formData, domain: e.target.value})}
                                isRequired
                                startContent={<Icon icon="solar:global-bold" width="18" height="18"/>}
                                classNames={{inputWrapper: "dark:bg-white/5 bg-black/5"}}
                            />

                            <div className="flex gap-2">
                                <Switch
                                    isSelected={formData.is_letsencrypt}
                                    onValueChange={(checked) => setFormData({...formData, is_letsencrypt: checked})}
                                >
                                    Use Let's Encrypt (automatic)
                                </Switch>
                            </div>

                            {!formData.is_letsencrypt && (
                                <div className="space-y-4 border-2 border-dashed border-default-200 rounded-lg p-4">
                                    <p className="text-sm font-semibold text-foreground/80">Upload Custom Certificate</p>

                                    <div>
                                        <label className="block text-sm mb-2">Certificate File (.crt or .pem)</label>
                                        <input
                                            type="file"
                                            onChange={(e) => setCertFile(e.target.files?.[0] || null)}
                                            accept=".crt,.pem"
                                            className="block w-full text-sm text-foreground/60
                        file:mr-4 file:py-2 file:px-4
                        file:rounded-lg file:border-0
                        file:text-sm file:font-semibold
                        file:bg-primary file:text-primary-foreground
                        hover:file:bg-primary/90 file:cursor-pointer"
                                        />
                                        {certFile && (
                                            <Chip className="mt-2" size="sm" color="success" variant="flat">
                                                {certFile.name}
                                            </Chip>
                                        )}
                                    </div>

                                    <div>
                                        <label className="block text-sm mb-2">Private Key File (.key or .pem) - Optional</label>
                                        <input
                                            type="file"
                                            onChange={(e) => setKeyFile(e.target.files?.[0] || null)}
                                            accept=".key,.pem"
                                            className="block w-full text-sm text-foreground/60
                        file:mr-4 file:py-2 file:px-4
                        file:rounded-lg file:border-0
                        file:text-sm file:font-semibold
                        file:bg-secondary file:text-secondary-foreground
                        hover:file:bg-secondary/90 file:cursor-pointer"
                                        />
                                        {keyFile && (
                                            <Chip className="mt-2" size="sm" color="success" variant="flat">
                                                {keyFile.name}
                                            </Chip>
                                        )}
                                    </div>
                                </div>
                            )}
                        </div>
                    </ModalBody>
                    <ModalFooter>
                        <Button variant="light" onPress={onCreateClose}>
                            Cancel
                        </Button>
                        <Button
                            color="primary"
                            onPress={handleCreateCertificate}
                            isDisabled={!formData.name || !formData.domain || (!formData.is_letsencrypt && !certFile)}
                            startContent={<Icon icon="solar:check-circle-bold" width="18" height="18"/>}
                        >
                            Add Certificate
                        </Button>
                    </ModalFooter>
                </ModalContent>
            </Modal>
        </>
    );
}
