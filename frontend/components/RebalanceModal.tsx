"use client";

import { useState } from "react";
import { PublicKey } from "@solana/web3.js";
import { YieldStrategy } from "@/lib/yieldAggregatorClient";

interface RebalanceModalProps {
  isOpen: boolean;
  onClose: () => void;
  strategies: YieldStrategy[];
  currentStrategy?: YieldStrategy;
  currentAmount?: number;
  onRebalance: (
    fromStrategy: PublicKey,
    toStrategy: PublicKey,
    amount: number,
  ) => Promise<void>;
}

export function RebalanceModal({
  isOpen,
  onClose,
  strategies,
  currentStrategy,
  currentAmount = 0,
  onRebalance,
}: RebalanceModalProps) {
  const [selectedStrategy, setSelectedStrategy] = useState<string>("");
  const [amount, setAmount] = useState<string>("");
  const [isLoading, setIsLoading] = useState(false);

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!currentStrategy || !selectedStrategy || !amount) return;

    setIsLoading(true);
    try {
      await onRebalance(
        currentStrategy.publicKey,
        new PublicKey(selectedStrategy),
        parseFloat(amount),
      );
      onClose();
      setSelectedStrategy("");
      setAmount("");
    } catch (error) {
      console.error("Rebalance failed:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const availableStrategies = strategies.filter(
    (s) =>
      s.publicKey.toString() !== currentStrategy?.publicKey.toString() &&
      s.isActive,
  );

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-xl font-bold text-gray-900 dark:text-white">
            Rebalance Position
          </h2>
          <button
            onClick={onClose}
            className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
          >
            ×
          </button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              From Strategy
            </label>
            <div className="p-3 bg-gray-100 dark:bg-gray-700 rounded-md">
              <div className="font-medium text-gray-900 dark:text-white">
                {currentStrategy?.name}
              </div>
              <div className="text-sm text-gray-600 dark:text-gray-400">
                Current Amount: {currentAmount.toFixed(2)} SOL
              </div>
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              To Strategy
            </label>
            <select
              value={selectedStrategy}
              onChange={(e) => setSelectedStrategy(e.target.value)}
              className="w-full p-3 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              required
            >
              <option value="">Select destination strategy</option>
              {availableStrategies.map((strategy) => (
                <option
                  key={strategy.publicKey.toString()}
                  value={strategy.publicKey.toString()}
                >
                  {strategy.name} (APY: {strategy.apy}%)
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Amount to Rebalance
            </label>
            <input
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="0.00"
              step="0.01"
              min="0"
              max={currentAmount}
              className="w-full p-3 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              required
            />
            <div className="flex justify-between mt-1">
              <span className="text-xs text-gray-500 dark:text-gray-400">
                Max: {currentAmount.toFixed(2)} SOL
              </span>
              <button
                type="button"
                onClick={() => setAmount(currentAmount.toString())}
                className="text-xs text-blue-600 dark:text-blue-400 hover:underline"
              >
                Use Max
              </button>
            </div>
          </div>

          <div className="flex space-x-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 py-2 px-4 border border-gray-300 dark:border-gray-600 rounded-md text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isLoading || !selectedStrategy || !amount}
              className="flex-1 py-2 px-4 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isLoading ? "Processing..." : "Rebalance"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
