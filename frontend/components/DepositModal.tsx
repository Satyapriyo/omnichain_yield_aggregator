import React, { useState } from "react";
import { YieldAggregatorClient } from "@/lib/yieldAggregatorClient";
import { Protocol } from "./YieldAggregatorDashboard";

interface DepositModalProps {
  protocol: Protocol;
  client: YieldAggregatorClient;
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

export const DepositModal: React.FC<DepositModalProps> = ({
  protocol,
  client,
  isOpen,
  onClose,
  onSuccess,
}) => {
  const [amount, setAmount] = useState("");
  const [minApy, setMinApy] = useState(protocol.currentApy.toString());
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setIsLoading(true);

    try {
      const amountNum = parseFloat(amount);
      const minApyNum = parseFloat(minApy);

      if (isNaN(amountNum) || amountNum <= 0) {
        throw new Error("Please enter a valid amount");
      }

      if (isNaN(minApyNum) || minApyNum < 0) {
        throw new Error("Please enter a valid minimum APY");
      }

      // For demo purposes, we'll use the first available strategy
      // In a real implementation, you'd map the protocol to the correct strategy
      const strategies = await client.getStrategies();
      const targetStrategy = strategies.find((s) =>
        s.name.toLowerCase().includes(protocol.name.toLowerCase()),
      );

      if (!targetStrategy) {
        throw new Error(`No strategy found for ${protocol.name}`);
      }

      await client.deposit(targetStrategy.publicKey, amountNum);

      onSuccess();
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : "An error occurred");
    } finally {
      setIsLoading(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full max-h-[90vh] overflow-y-auto">
        <div className="p-6">
          {/* Header */}
          <div className="flex justify-between items-center mb-6">
            <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
              Deposit to {protocol.name}
            </h2>
            <button
              onClick={onClose}
              className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
            >
              ✕
            </button>
          </div>

          {/* Protocol Info */}
          <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 mb-6">
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <p className="text-gray-600 dark:text-gray-300">Chain</p>
                <p className="font-semibold text-gray-900 dark:text-white">
                  {protocol.chainId === 40168
                    ? "Solana"
                    : `Chain ${protocol.chainId}`}
                </p>
              </div>
              <div>
                <p className="text-gray-600 dark:text-gray-300">Current APY</p>
                <p className="font-semibold text-green-600 dark:text-green-400">
                  {(protocol.currentApy / 100).toFixed(2)}%
                </p>
              </div>
              <div>
                <p className="text-gray-600 dark:text-gray-300">Risk Score</p>
                <p className="font-semibold text-gray-900 dark:text-white">
                  {protocol.riskScore}/10
                </p>
              </div>
              <div>
                <p className="text-gray-600 dark:text-gray-300">
                  Available Capacity
                </p>
                <p className="font-semibold text-gray-900 dark:text-white">
                  $
                  {((protocol.maxCapacity - protocol.tvl) / 1000000).toFixed(1)}
                  M
                </p>
              </div>
            </div>
          </div>

          {/* Form */}
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Deposit Amount (USDC)
              </label>
              <input
                type="number"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                placeholder="0.00"
                min="0"
                step="0.01"
                className="input-field"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Minimum APY (%)
              </label>
              <input
                type="number"
                value={minApy}
                onChange={(e) => setMinApy(e.target.value)}
                placeholder="0.00"
                min="0"
                step="0.01"
                className="input-field"
                required
              />
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                Transaction will fail if APY drops below this threshold
              </p>
            </div>

            {error && (
              <div className="bg-red-50 dark:bg-red-900 border border-red-200 dark:border-red-700 rounded-md p-3">
                <p className="text-sm text-red-600 dark:text-red-400">
                  {error}
                </p>
              </div>
            )}

            {/* Action Buttons */}
            <div className="flex gap-3 pt-4">
              <button
                type="button"
                onClick={onClose}
                className="flex-1 btn-secondary"
                disabled={isLoading}
              >
                Cancel
              </button>
              <button
                type="submit"
                className="flex-1 btn-primary"
                disabled={isLoading || !amount}
              >
                {isLoading ? "Processing..." : "Deposit"}
              </button>
            </div>
          </form>

          {/* Info */}
          <div className="mt-6 p-4 bg-blue-50 dark:bg-blue-900 rounded-lg">
            <p className="text-sm text-blue-800 dark:text-blue-200">
              💡 <strong>Note:</strong> Cross-chain deposits may take several
              minutes to process. You'll receive a confirmation once the
              transaction is complete.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
