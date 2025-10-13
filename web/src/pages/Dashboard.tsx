import { useEffect, useState } from "react";
import { useDisclosure } from "@heroui/react";
import { motion } from "framer-motion";
import { metricsAPI, serverAPI } from "../utils/api";
import { Server, SystemStats } from "../types";
import CertificateModal from "../components/CertificateModal";
import QuickStats from "../components/QuickStats";
import SystemResources from "../components/SystemResources";
import ServersTable from "../components/ServersTable";
import CreateServerModal from "../components/CreateServerModal";

export default function Dashboard() {
  const { isOpen: isCertOpen, onOpen: onCertOpen, onClose: onCertClose } = useDisclosure();
  const { isOpen: isCreateOpen, onOpen: onCreateOpen, onClose: onCreateClose } = useDisclosure();
  const [stats, setStats] = useState<SystemStats | null>(null);
  const [servers, setServers] = useState<Server[]>([]);
  const [, setLoading] = useState(true);

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadData = async () => {
    try {
      const [statsData, serversData] = await Promise.all([
        metricsAPI.getSystemStats(),
        serverAPI.list(),
      ]);
      setStats(statsData);
      setServers(serversData);
    } catch (error) {
      console.error("Failed to load data:", error);
    } finally {
      setLoading(false);
    }
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
      await loadData();
    } catch (error: any) {
      alert("Action failed: " + error.message);
    }
  };

  return (
    <>
      <div className="p-6 space-y-6">
        <motion.div
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
        >
          <h1 className="text-3xl font-bold mb-2">Dashboard</h1>
          <p className="text-foreground/60">Monitor your minipx proxy servers and system resources</p>
        </motion.div>

        {/* Quick Stats */}
        <QuickStats servers={servers} />

        {/* System Resources */}
        <SystemResources stats={stats} />

        {/* Servers Table */}
        <ServersTable
          servers={servers}
          onCreateOpen={onCreateOpen}
          onCertOpen={onCertOpen}
          onServerAction={handleServerAction}
        />
      </div>

      {/* Modals */}
      <CreateServerModal
        isOpen={isCreateOpen}
        onClose={onCreateClose}
        onServerCreated={loadData}
      />
      <CertificateModal isOpen={isCertOpen} onClose={onCertClose} />
    </>
  );
}
