"use client";

import React, { useState, useEffect } from "react";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import {
  YieldAggregatorClient,
  YieldStrategy,
} from "@/lib/yieldAggregatorClient";
import { ProtocolCard } from "./ProtocolCard";
import { UserPositionCard } from "./UserPositionCard";
import { DepositModal } from "./DepositModal";
import { WithdrawModal } from "./WithdrawModal";
import { RebalanceModal } from "./RebalanceModal";
import { AdminPanel } from "./AdminPanel";

export interface Protocol {
  name: string;
  chainId: number;
  currentApy: number;
  tvl: number;
  maxCapacity: number;
  riskScore: number;
  isActive: boolean;
  lastUpdate: number;
}

export interface UserPosition {
  user: string;
  totalDeposits: number;
  totalYieldEarned: number;
  positionCount: number;
  lastActivity: number;
}

export const YieldAggregatorDashboard: React.FC = () => {
  const { connection } = useConnection();
  const { publicKey, connected } = useWallet();
  const [client, setClient] = useState<YieldAggregatorClient | null>(null);

  // State
  const [protocols, setProtocols] = useState<Protocol[]>([]);
  const [strategies, setStrategies] = useState<YieldStrategy[]>([]);
  const [userPosition, setUserPosition] = useState<UserPosition | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  // Modal states
  const [showDepositModal, setShowDepositModal] = useState(false);
  const [showWithdrawModal, setShowWithdrawModal] = useState(false);
  const [showRebalanceModal, setShowRebalanceModal] = useState(false);
  const [showAdminPanel, setShowAdminPanel] = useState(false);

  // Selected protocol for modals
  const [selectedProtocol, setSelectedProtocol] = useState<Protocol | null>(
    null,
  );

  // Initialize client when wallet connects
  useEffect(() => {
    if (connection) {
      const yieldClient = new YieldAggregatorClient(connection);
      yieldClient.setWallet(publicKey);
      setClient(yieldClient);
    } else {
      setClient(null);
    }
  }, [connection, publicKey, connected]);

  // Load data when client is available
  useEffect(() => {
    if (client) {
      loadData();
    }
  }, [client]);

  const loadData = async () => {
    if (!client) return;

    setIsLoading(true);
    try {
      // Fetch strategies from the client
      const fetchedStrategies = await client.getStrategies();
      setStrategies(fetchedStrategies);

      // Mock data for now - you can implement actual data fetching
      const mockProtocols: Protocol[] = [
        {
          name: "Compound",
          chainId: 1,
          currentApy: 850, // 8.5%
          tvl: 1500000,
          maxCapacity: 5000000,
          riskScore: 3,
          isActive: true,
          lastUpdate: Date.now(),
        },
        {
          name: "Aave",
          chainId: 1,
          currentApy: 720, // 7.2%
          tvl: 2200000,
          maxCapacity: 8000000,
          riskScore: 2,
          isActive: true,
          lastUpdate: Date.now(),
        },
        {
          name: "Solana Lido",
          chainId: 40168,
          currentApy: 920, // 9.2%
          tvl: 800000,
          maxCapacity: 3000000,
          riskScore: 4,
          isActive: true,
          lastUpdate: Date.now(),
        },
      ];

      setProtocols(mockProtocols);

      // Mock user position
      const mockUserPosition: UserPosition = {
        user: publicKey?.toString() || "",
        totalDeposits: 5000,
        totalYieldEarned: 125,
        positionCount: 2,
        lastActivity: Date.now(),
      };

      setUserPosition(mockUserPosition);
    } catch (error) {
      console.error("Error loading data:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleDeposit = (protocol: Protocol) => {
    setSelectedProtocol(protocol);
    setShowDepositModal(true);
  };

  const handleWithdraw = () => {
    setShowWithdrawModal(true);
  };

  const handleRebalance = () => {
    setShowRebalanceModal(true);
  };

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat("en-US", {
      style: "currency",
      currency: "USD",
      minimumFractionDigits: 0,
      maximumFractionDigits: 0,
    }).format(amount);
  };

  const formatApy = (apy: number) => {
    return (apy / 100).toFixed(2) + "%";
  };

  if (!connected) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] text-center">
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8 max-w-md">
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">
            Connect Your Wallet
          </h2>
          <p className="text-gray-600 dark:text-gray-300 mb-6">
            Please connect your Solana wallet to access the Yield Aggregator
            dashboard.
          </p>
          <div className="text-4xl mb-4">🔗</div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      {/* User Position Summary */}
      {userPosition && (
        <UserPositionCard
          position={userPosition}
          onWithdraw={handleWithdraw}
          onRebalance={handleRebalance}
          formatCurrency={formatCurrency}
        />
      )}

      {/* Admin Panel Toggle */}
      <div className="flex justify-end">
        <button
          onClick={() => setShowAdminPanel(!showAdminPanel)}
          className="btn-secondary text-sm"
        >
          {showAdminPanel ? "Hide Admin Panel" : "Show Admin Panel"}
        </button>
      </div>

      {/* Admin Panel */}
      {showAdminPanel && (
        <AdminPanel strategies={strategies} onRefresh={loadData} />
      )}

      {/* Protocols Grid */}
      <div>
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
            Available Protocols
          </h2>
          <button
            onClick={loadData}
            disabled={isLoading}
            className="btn-secondary"
          >
            {isLoading ? "Loading..." : "Refresh"}
          </button>
        </div>

        {isLoading ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {[1, 2, 3].map((i) => (
              <div key={i} className="card animate-pulse">
                <div className="h-4 bg-gray-300 rounded w-3/4 mb-4"></div>
                <div className="h-3 bg-gray-300 rounded w-1/2 mb-2"></div>
                <div className="h-3 bg-gray-300 rounded w-2/3"></div>
              </div>
            ))}
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {protocols.map((protocol) => (
              <ProtocolCard
                key={`${protocol.name}-${protocol.chainId}`}
                protocol={protocol}
                onDeposit={() => handleDeposit(protocol)}
                formatCurrency={formatCurrency}
                formatApy={formatApy}
              />
            ))}
          </div>
        )}

        {!isLoading && protocols.length === 0 && (
          <div className="text-center py-12">
            <div className="text-6xl mb-4">📊</div>
            <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
              No Protocols Available
            </h3>
            <p className="text-gray-600 dark:text-gray-300">
              No yield protocols have been added yet. Check back later or
              contact an administrator.
            </p>
          </div>
        )}
      </div>

      {/* Modals */}
      {showDepositModal && selectedProtocol && client && (
        <DepositModal
          protocol={selectedProtocol}
          client={client}
          isOpen={showDepositModal}
          onClose={() => setShowDepositModal(false)}
          onSuccess={loadData}
        />
      )}

      {showWithdrawModal && client && (
        <WithdrawModal
          client={client}
          isOpen={showWithdrawModal}
          onClose={() => setShowWithdrawModal(false)}
          onSuccess={loadData}
          userPosition={userPosition}
        />
      )}

      {showRebalanceModal && client && (
        <RebalanceModal
          strategies={strategies}
          isOpen={showRebalanceModal}
          onClose={() => setShowRebalanceModal(false)}
          onRebalance={async (fromStrategy, toStrategy, amount) => {
            try {
              await client.rebalance(fromStrategy, toStrategy, amount);
              loadData();
            } catch (error) {
              console.error("Rebalance failed:", error);
            }
          }}
        />
      )}
    </div>
  );
};
