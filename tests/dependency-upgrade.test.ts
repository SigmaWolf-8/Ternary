/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 */
import { describe, expect, it } from "vitest";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { AccountId, Hbar, Transaction, TransactionId, TransferTransaction } from "@hashgraph/sdk";

describe("Patched dependency compatibility", () => {
  it("returns structured form errors through the Zod 4 resolver", async () => {
    const resolver = zodResolver(z.object({ email: z.string().email() }));
    const result = await resolver(
      { email: "not-an-email" },
      {},
      { fields: {}, shouldUseNativeValidation: false, criteriaMode: "all" },
    );
    expect(result.errors.email?.message).toBeTruthy();
    expect(result.values).toEqual({});
  });

  it("retains string-keyed record validation and detailed issues", () => {
    const schema = z.record(z.string(), z.string());
    expect(schema.parse({ source: "test" })).toEqual({ source: "test" });
    const result = schema.safeParse({ source: 123 });
    expect(result.success).toBe(false);
    if (!result.success) expect(result.error.issues[0].path).toEqual(["source"]);
  });

  it("round-trips Hedera protobuf bytes offline without submitting a transaction", () => {
    const transaction = new TransferTransaction()
      .setTransactionId(TransactionId.generate("0.0.1001"))
      .setNodeAccountIds([AccountId.fromString("0.0.3")])
      .addHbarTransfer("0.0.1001", new Hbar(-1))
      .addHbarTransfer("0.0.1002", new Hbar(1))
      .freeze();
    const bytes = transaction.toBytes();
    const decoded = Transaction.fromBytes(bytes);
    expect(decoded).toBeInstanceOf(TransferTransaction);
    expect(decoded.toBytes()).toEqual(bytes);
  });
});
