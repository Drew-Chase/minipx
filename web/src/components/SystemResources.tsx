import { Card, CardBody, CardHeader, Chip, Progress } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { motion } from "framer-motion";
import { SystemStats } from "../types";

interface SystemResourcesProps {
  stats: SystemStats | null;
}

export default function SystemResources({ stats }: SystemResourcesProps) {
  if (!stats) return null;

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
  };

  return (
    <motion.div
      className="grid grid-cols-1 md:grid-cols-2 gap-6"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5, delay: 0.4 }}
    >
      {/* CPU Usage */}
      <Card className="bg-blue-500/5">
        <CardHeader className="flex gap-3">
          <Icon icon="solar:cpu-bold" width="24" height="24" className="text-primary" />
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

      {/* Memory Usage */}
      <Card className="bg-blue-500/5">
        <CardHeader className="flex gap-3">
          <Icon icon="solar:chip-bold" width="24" height="24" className="text-secondary" />
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

      {/* Disk Usage */}
      <Card className="bg-blue-500/5">
        <CardHeader className="flex gap-3">
          <Icon icon="solar:database-bold" width="24" height="24" className="text-warning" />
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

      {/* Network */}
      <Card className="bg-blue-500/5">
        <CardHeader className="flex gap-3">
          <Icon icon="solar:wifi-router-bold" width="24" height="24" className="text-success" />
          <div>
            <p className="text-lg font-semibold">Network</p>
            <p className="text-sm text-foreground/60">Data transfer</p>
          </div>
        </CardHeader>
        <CardBody className="space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Icon icon="solar:download-bold" width="18" height="18" className="text-success" />
              <span className="text-sm">Download</span>
            </div>
            <span className="font-semibold">{formatBytes(stats.network_in)}</span>
          </div>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Icon icon="solar:upload-bold" width="18" height="18" className="text-primary" />
              <span className="text-sm">Upload</span>
            </div>
            <span className="font-semibold">{formatBytes(stats.network_out)}</span>
          </div>
        </CardBody>
      </Card>
    </motion.div>
  );
}
