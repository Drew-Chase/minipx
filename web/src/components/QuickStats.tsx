import { Card, CardBody } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { motion } from "framer-motion";
import { Server } from "../types";

interface QuickStatsProps {
  servers: Server[];
}

export default function QuickStats({ servers }: QuickStatsProps) {
  const statusCounts = servers.reduce((acc, server) => {
    acc[server.status] = (acc[server.status] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  const stats = [
    {
      icon: "solar:server-bold",
      label: "Total Servers",
      value: servers.length,
      color: "primary",
      delay: "0s",
    },
    {
      icon: "solar:check-circle-bold",
      label: "Running",
      value: statusCounts.running || 0,
      color: "success",
      delay: "0.1s",
    },
    {
      icon: "solar:close-circle-bold",
      label: "Stopped",
      value: statusCounts.stopped || 0,
      color: "danger",
      delay: "0.2s",
    },
    {
      icon: "solar:shield-keyhole-bold",
      label: "SSL Enabled",
      value: servers.filter((s) => s.ssl_enabled).length,
      color: "warning",
      delay: "0.3s",
    },
  ];

  return (
    <motion.div
      className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.5, delay: 0.1 }}
    >
      {stats.map((stat, index) => (
        <Card
          key={index}
          className="animate-scale-in hover:scale-105 transition-transform bg-blue-500/5"
          style={{ animationDelay: stat.delay }}
        >
          <CardBody className="flex flex-row items-center gap-4">
            <div className={`p-3 rounded-full bg-${stat.color}/10`}>
              <Icon icon={stat.icon} width="24" height="24" className={`text-${stat.color}`} />
            </div>
            <div>
              <p className="text-sm text-foreground/60">{stat.label}</p>
              <p className="text-2xl font-bold">{stat.value}</p>
            </div>
          </CardBody>
        </Card>
      ))}
    </motion.div>
  );
}
