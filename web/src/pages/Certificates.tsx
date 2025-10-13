import {useEffect, useState} from "react";
import {Button, Card, CardBody, CardHeader, Chip, Input, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader, Switch, useDisclosure} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {certificateAPI} from "../utils/api";
import {Certificate} from "../types";
import {motion} from "framer-motion";

export default function Certificates()
{
    const [certificates, setCertificates] = useState<Certificate[]>([]);
    const [, setLoading] = useState(true);
    const {isOpen, onOpen, onClose} = useDisclosure();

    const [formData, setFormData] = useState({
        name: "",
        domain: "",
        is_letsencrypt: true
    });
    const [certFile, setCertFile] = useState<File | null>(null);
    const [keyFile, setKeyFile] = useState<File | null>(null);

    useEffect(() =>
    {
        loadCertificates();
    }, []);

    const loadCertificates = async () =>
    {
        try
        {
            const data = await certificateAPI.list();
            setCertificates(data);
        } catch (error)
        {
            console.error("Failed to load certificates:", error);
        } finally
        {
            setLoading(false);
        }
    };

    const handleCreateCertificate = async () =>
    {
        try
        {
            const cert = await certificateAPI.create(formData);

            // Upload certificate files if provided
            if (!formData.is_letsencrypt && certFile)
            {
                await certificateAPI.uploadCertificate(cert.id, certFile, keyFile || undefined);
            }

            await loadCertificates();
            onClose();
            resetForm();
        } catch (error: any)
        {
            alert("Failed to create certificate: " + error.message);
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

    const handleDelete = async (id: string) =>
    {
        if (confirm("Delete this certificate?"))
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

    return (
        <div className="p-6 space-y-6">
            <motion.div
                className="flex justify-between items-center"
                initial={{opacity: 0, y: -20}}
                animate={{opacity: 1, y: 0}}
            >
                <div>
                    <h1 className="text-3xl font-bold mb-2">SSL Certificates</h1>
                    <p className="text-foreground/60">Manage SSL/TLS certificates for your domains</p>
                </div>
                <Button
                    color="primary"
                    size="lg"
                    startContent={<Icon icon="solar:add-circle-bold" width="20" height="20"/>}
                    onClick={onOpen}
                    className="hover:scale-105 transition-transform"
                >
                    Add Certificate
                </Button>
            </motion.div>

            <motion.div
                className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"
                initial={{opacity: 0}}
                animate={{opacity: 1}}
                transition={{delay: 0.2}}
            >
                {certificates.map((cert, index) => (
                    <motion.div
                        key={cert.id}
                        initial={{opacity: 0, scale: 0.9}}
                        animate={{opacity: 1, scale: 1}}
                        transition={{delay: index * 0.05}}
                    >
                        <Card className="hover:scale-105 transition-all duration-200 hover:shadow-lg">
                            <CardHeader className="flex justify-between">
                                <div className="flex items-center gap-3">
                                    <div className={`p-2 rounded-lg ${cert.is_letsencrypt ? "bg-success/10" : "bg-primary/10"}`}>
                                        <Icon icon="solar:shield-keyhole-bold" width="24" height="24" className={cert.is_letsencrypt ? "text-success" : "text-primary"}/>
                                    </div>
                                    <div>
                                        <p className="font-semibold text-lg">{cert.name}</p>
                                        <p className="text-sm text-foreground/60">{cert.domain}</p>
                                    </div>
                                </div>
                                <Button
                                    isIconOnly
                                    size="sm"
                                    color="danger"
                                    variant="light"
                                    onClick={() => handleDelete(cert.id)}
                                >
                                    <Icon icon="solar:trash-bin-trash-bold" width="18" height="18"/>
                                </Button>
                            </CardHeader>

                            <CardBody className="space-y-2">
                                <Chip
                                    size="sm"
                                    variant="flat"
                                    color={cert.is_letsencrypt ? "success" : "primary"}
                                    startContent={<Icon icon={cert.is_letsencrypt ? "solar:bolt-bold" : "solar:document-bold"} width="14" height="14"/>}
                                >
                                    {cert.is_letsencrypt ? "Let's Encrypt" : "Custom"}
                                </Chip>

                                {cert.expiry_date && (
                                    <div className="flex items-center gap-2 text-sm">
                                        <Icon icon="solar:calendar-bold" width="16" height="16" className="text-foreground/60"/>
                                        <span className="text-foreground/80">
                      Expires: {new Date(cert.expiry_date).toLocaleDateString()}
                    </span>
                                    </div>
                                )}

                                <div className="flex items-center gap-2 text-sm">
                                    <Icon icon="solar:clock-circle-bold" width="16" height="16" className="text-foreground/60"/>
                                    <span className="text-foreground/80">
                    Created: {new Date(cert.created_at).toLocaleDateString()}
                  </span>
                                </div>
                            </CardBody>
                        </Card>
                    </motion.div>
                ))}
            </motion.div>

            {/* Create Certificate Modal */}
            <Modal isOpen={isOpen} onClose={onClose} size="xl">
                <ModalContent>
                    <ModalHeader className="flex gap-2 items-center">
                        <Icon icon="solar:shield-keyhole-bold" width="24" height="24" className="text-primary"/>
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
                            />

                            <Input
                                label="Domain"
                                placeholder="example.com"
                                value={formData.domain}
                                onChange={(e) => setFormData({...formData, domain: e.target.value})}
                                isRequired
                                startContent={<Icon icon="solar:global-bold" width="18" height="18"/>}
                            />

                            <Switch
                                isSelected={formData.is_letsencrypt}
                                onValueChange={(checked) => setFormData({...formData, is_letsencrypt: checked})}
                            >
                                Use Let's Encrypt (Automatic)
                            </Switch>

                            {!formData.is_letsencrypt && (
                                <>
                                    <div className="border-2 border-dashed border-default-200 rounded-lg p-4 text-center hover:border-primary transition-colors">
                                        <input
                                            type="file"
                                            id="cert-upload"
                                            className="hidden"
                                            onChange={(e) => setCertFile(e.target.files?.[0] || null)}
                                            accept=".pem,.crt,.cer"
                                        />
                                        <label htmlFor="cert-upload" className="cursor-pointer">
                                            <Icon icon="solar:document-bold" width="36" height="36" className="mx-auto text-primary mb-2"/>
                                            <p className="font-semibold">Certificate File (.pem)</p>
                                            <p className="text-sm text-foreground/60">Required</p>
                                            {certFile && (
                                                <Chip className="mt-2" color="success" variant="flat">
                                                    {certFile.name}
                                                </Chip>
                                            )}
                                        </label>
                                    </div>

                                    <div className="border-2 border-dashed border-default-200 rounded-lg p-4 text-center hover:border-primary transition-colors">
                                        <input
                                            type="file"
                                            id="key-upload"
                                            className="hidden"
                                            onChange={(e) => setKeyFile(e.target.files?.[0] || null)}
                                            accept=".pem,.key"
                                        />
                                        <label htmlFor="key-upload" className="cursor-pointer">
                                            <Icon icon="solar:key-bold" width="36" height="36" className="mx-auto text-warning mb-2"/>
                                            <p className="font-semibold">Private Key (.pem)</p>
                                            <p className="text-sm text-foreground/60">Optional</p>
                                            {keyFile && (
                                                <Chip className="mt-2" color="success" variant="flat">
                                                    {keyFile.name}
                                                </Chip>
                                            )}
                                        </label>
                                    </div>
                                </>
                            )}
                        </div>
                    </ModalBody>
                    <ModalFooter>
                        <Button variant="light" onPress={onClose}>
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
        </div>
    );
}
