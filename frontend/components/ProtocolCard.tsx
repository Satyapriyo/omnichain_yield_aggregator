import React from "react";
import { Protocol } from "./YieldAggregatorDashboard";

interface ProtocolCardProps {
  protocol: Protocol;
  onDeposit: () => void;
  formatCurrency: (amount: number) => string;
  formatApy: (apy: number) => string;
}

export const ProtocolCard: React.FC<ProtocolCardProps> = ({
  protocol,
  onDeposit,
  formatCurrency,
  formatApy,
}) => {
  const getRiskColor = (riskScore: number) => {
    if (riskScore <= 3) return "text-green-600";
    if (riskScore <= 6) return "text-yellow-600";
    return "text-red-600";
  };

  const getRiskLabel = (riskScore: number) => {
    if (riskScore <= 3) return "Low Risk";
    if (riskScore <= 6) return "Medium Risk";
    return "High Risk";
  };

  const getChainName = (chainId: number) => {
    switch (chainId) {
      case 1:
        return "Ethereum";
      case 137:
        return "Polygon";
      case 56:
        return "BSC";
      case 43114:
        return "Avalanche";
      case 40168:
        return "Solana";
      default:
        return `Chain ${chainId}`;
    }
  };

  const utilizationRate = (protocol.tvl / protocol.maxCapacity) * 100;

  return (
    <div className="card hover:shadow-xl transition-shadow duration-300">
      {/* Header */}
      <div className="flex justify-between items-start mb-4">
        <div>
          <h3 className="text-xl font-bold text-gray-900 dark:text-white">
            {protocol.name}
          </h3>
          <p className="text-sm text-gray-600 dark:text-gray-300">
            {getChainName(protocol.chainId)}
          </p>
        </div>
        <div
          className={`px-2 py-1 rounded-full text-xs font-medium ${
            protocol.isActive
              ? "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
              : "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200"
          }`}
        >
          {protocol.isActive ? "Active" : "Inactive"}
        </div>
      </div>

      {/* APY */}
      <div className="mb-4">
        <div className="flex items-baseline">
          <span className="text-3xl font-bold text-blue-600 dark:text-blue-400">
            {formatApy(protocol.currentApy)}
          </span>
          <span className="ml-2 text-sm text-gray-600 dark:text-gray-300">
            APY
          </span>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-2 gap-4 mb-4">
        <div>
          <p className="text-sm text-gray-600 dark:text-gray-300 mb-1">TVL</p>
          <p className="font-semibold text-gray-900 dark:text-white">
            {formatCurrency(protocol.tvl)}
          </p>
        </div>
        <div>
          <p className="text-sm text-gray-600 dark:text-gray-300 mb-1">Risk</p>
          <p className={`font-semibold ${getRiskColor(protocol.riskScore)}`}>
            {getRiskLabel(protocol.riskScore)}
          </p>
        </div>
      </div>

      {/* Capacity Bar */}
      <div className="mb-4">
        <div className="flex justify-between text-sm text-gray-600 dark:text-gray-300 mb-2">
          <span>Utilization</span>
          <span>{utilizationRate.toFixed(1)}%</span>
        </div>
        <div className="w-full bg-gray-200 rounded-full h-2 dark:bg-gray-700">
          <div
            className={`h-2 rounded-full transition-all duration-300 ${
              utilizationRate > 80
                ? "bg-red-500"
                : utilizationRate > 60
                  ? "bg-yellow-500"
                  : "bg-green-500"
            }`}
            style={{ width: `${Math.min(utilizationRate, 100)}%` }}
          ></div>
        </div>
        <div className="flex justify-between text-xs text-gray-500 dark:text-gray-400 mt-1">
          <span>{formatCurrency(protocol.tvl)}</span>
          <span>{formatCurrency(protocol.maxCapacity)}</span>
        </div>
      </div>

      {/* Action Button */}
      <button
        onClick={onDeposit}
        disabled={!protocol.isActive || utilizationRate >= 100}
        className="w-full btn-primary disabled:bg-gray-400 disabled:cursor-not-allowed"
      >
        {!protocol.isActive
          ? "Protocol Inactive"
          : utilizationRate >= 100
            ? "At Capacity"
            : "Deposit"}
      </button>

      {/* Last Update */}
      <p className="text-xs text-gray-500 dark:text-gray-400 mt-2 text-center">
        Last updated: {new Date(protocol.lastUpdate).toLocaleString()}
      </p>
    </div>
  );
};
