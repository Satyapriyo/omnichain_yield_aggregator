"use client";

import { useState, useEffect } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { YieldStrategy } from "@/lib/yieldAggregatorClient";

interface AdminPanelProps {
  strategies: YieldStrategy[];
  onRefresh: () => void;
}

export function AdminPanel({ strategies, onRefresh }: AdminPanelProps) {
  const { publicKey } = useWallet();
  const [isAdmin, setIsAdmin] = useState(false);
  const [selectedStrategy, setSelectedStrategy] = useState<string>("");
  const [isToggling, setIsToggling] = useState(false);

  // Simple admin check - in production you'd check against a list of admin addresses
  useEffect(() => {
    if (publicKey) {
      // For demo purposes, any connected wallet can be admin
      setIsAdmin(true);
    } else {
      setIsAdmin(false);
    }
  }, [publicKey]);

  const handleToggleStrategy = async (
    strategyPubkey: string,
    currentStatus: boolean,
  ) => {
    setIsToggling(true);
    try {
      // Simulate API call to toggle strategy
      await new Promise((resolve) => setTimeout(resolve, 1000));
      console.log(
        `Toggling strategy ${strategyPubkey} from ${currentStatus} to ${!currentStatus}`,
      );
      onRefresh();
    } catch (error) {
      console.error("Failed to toggle strategy:", error);
    } finally {
      setIsToggling(false);
    }
  };

  const handleEmergencyPause = async () => {
    if (
      !confirm(
        "Are you sure you want to pause all strategies? This will stop new deposits.",
      )
    ) {
      return;
    }

    try {
      // Simulate emergency pause
      await new Promise((resolve) => setTimeout(resolve, 2000));
      console.log("Emergency pause activated");
      onRefresh();
    } catch (error) {
      console.error("Failed to pause strategies:", error);
    }
  };

  if (!isAdmin) {
    return (
      <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
        <div className="flex items-center">
          <div className="flex-shrink-0">
            <svg
              className="h-5 w-5 text-yellow-400"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fillRule="evenodd"
                d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                clipRule="evenodd"
              />
            </svg>
          </div>
          <div className="ml-3">
            <p className="text-sm text-yellow-800 dark:text-yellow-200">
              Connect wallet to access admin features
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-bold text-gray-900 dark:text-white">
          Admin Panel
        </h2>
        <div className="flex items-center space-x-2">
          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
            Admin
          </span>
        </div>
      </div>

      <div className="space-y-6">
        {/* Strategy Management */}
        <div>
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
            Strategy Management
          </h3>
          <div className="space-y-3">
            {strategies.map((strategy) => (
              <div
                key={strategy.publicKey.toString()}
                className="flex items-center justify-between p-4 border border-gray-200 dark:border-gray-700 rounded-lg"
              >
                <div className="flex-1">
                  <div className="flex items-center space-x-3">
                    <span className="font-medium text-gray-900 dark:text-white">
                      {strategy.name}
                    </span>
                    <span
                      className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                        strategy.isActive
                          ? "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
                          : "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200"
                      }`}
                    >
                      {strategy.isActive ? "Active" : "Inactive"}
                    </span>
                  </div>
                  <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                    APY: {strategy.apy}% | TVL: ${strategy.tvl.toLocaleString()}{" "}
                    | Risk: {strategy.riskLevel}
                  </div>
                </div>
                <button
                  onClick={() =>
                    handleToggleStrategy(
                      strategy.publicKey.toString(),
                      strategy.isActive,
                    )
                  }
                  disabled={isToggling}
                  className={`px-4 py-2 rounded-md text-sm font-medium ${
                    strategy.isActive
                      ? "bg-red-600 text-white hover:bg-red-700"
                      : "bg-green-600 text-white hover:bg-green-700"
                  } disabled:opacity-50 disabled:cursor-not-allowed`}
                >
                  {isToggling
                    ? "Processing..."
                    : strategy.isActive
                      ? "Pause"
                      : "Activate"}
                </button>
              </div>
            ))}
          </div>
        </div>

        {/* Emergency Controls */}
        <div>
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
            Emergency Controls
          </h3>
          <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <div>
                <h4 className="text-sm font-medium text-red-800 dark:text-red-200">
                  Emergency Pause
                </h4>
                <p className="text-sm text-red-600 dark:text-red-300 mt-1">
                  Immediately pause all strategies to prevent new deposits
                </p>
              </div>
              <button
                onClick={handleEmergencyPause}
                className="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 text-sm font-medium"
              >
                Emergency Pause
              </button>
            </div>
          </div>
        </div>

        {/* Protocol Stats */}
        <div>
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
            Protocol Statistics
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
              <div className="text-sm text-gray-600 dark:text-gray-400">
                Total Strategies
              </div>
              <div className="text-2xl font-bold text-gray-900 dark:text-white">
                {strategies.length}
              </div>
            </div>
            <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
              <div className="text-sm text-gray-600 dark:text-gray-400">
                Active Strategies
              </div>
              <div className="text-2xl font-bold text-green-600">
                {strategies.filter((s) => s.isActive).length}
              </div>
            </div>
            <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
              <div className="text-sm text-gray-600 dark:text-gray-400">
                Total TVL
              </div>
              <div className="text-2xl font-bold text-blue-600">
                $
                {strategies.reduce((sum, s) => sum + s.tvl, 0).toLocaleString()}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
