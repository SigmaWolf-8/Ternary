export type Trit = -1 | 0 | 1;

export interface ToroidalAddress {
  eta: number;
  theta: number;
  psi: number;
}

export interface NeighborEntry {
  id: string;
  address: ToroidalAddress;
  dominantAxis: 'eta' | 'theta' | 'psi';
}

export interface TernaryGradient {
  eta: Trit;
  theta: Trit;
  psi: Trit;
}

export const R_NETWORK = 10.0;

export function angleDiff(a: number, b: number): number {
  let d = ((a - b) % (2 * Math.PI) + 3 * Math.PI) % (2 * Math.PI) - Math.PI;
  return d;
}

export function toroidalDistance(
  a: ToroidalAddress,
  b: ToroidalAddress,
  R: number = R_NETWORK
): number {
  const dPsi = angleDiff(a.psi, b.psi);
  const dTheta = angleDiff(a.theta, b.theta);
  const dEta = Math.abs(a.eta - b.eta);
  return Math.sqrt(
    (R + a.eta * Math.cos(a.theta)) ** 2 * dPsi ** 2
    + dEta ** 2
    + a.eta ** 2 * dTheta ** 2
  );
}

export function naturalNeighbors(
  node: ToroidalAddress,
  allNodes: { id: string; address: ToroidalAddress }[],
  k: number,
  R: number = R_NETWORK
): NeighborEntry[] {
  return allNodes
    .map(n => {
      const dist = toroidalDistance(node, n.address, R);
      const d = {
        eta: Math.abs(n.address.eta - node.eta),
        theta: Math.abs(angleDiff(n.address.theta, node.theta)),
        psi: Math.abs(angleDiff(n.address.psi, node.psi)),
      };
      let dominantAxis: 'eta' | 'theta' | 'psi' = 'eta';
      if (d.theta > d.eta && d.theta > d.psi) dominantAxis = 'theta';
      else if (d.psi > d.eta) dominantAxis = 'psi';

      return { id: n.id, address: n.address, dominantAxis, dist };
    })
    .filter(n => n.dist > 0)
    .sort((a, b) => a.dist - b.dist)
    .slice(0, k)
    .map(({ id, address, dominantAxis }) => ({ id, address, dominantAxis }));
}

export function gf3Sub(a: Trit, b: Trit): Trit {
  const raw = ((a - b) % 3 + 3) % 3;
  return (raw === 2 ? -1 : raw) as Trit;
}

export function gf3Add(a: Trit, b: Trit): Trit {
  const raw = ((a + b) % 3 + 3) % 3;
  return (raw === 2 ? -1 : raw) as Trit;
}

export function gf3Neg(a: Trit): Trit {
  return (-a) as Trit;
}

export function ternaryGradient(
  localValue: Trit,
  neighbors: { id: string; fieldValue: Trit; dominantAxis: 'eta' | 'theta' | 'psi' }[]
): TernaryGradient {
  const byAxis: Record<string, Trit[]> = { eta: [], theta: [], psi: [] };
  for (const n of neighbors) {
    const diff = gf3Neg(gf3Sub(n.fieldValue, localValue));
    byAxis[n.dominantAxis].push(diff);
  }
  return {
    eta: majorityVote(byAxis.eta),
    theta: majorityVote(byAxis.theta),
    psi: majorityVote(byAxis.psi),
  };
}

export function majorityVote(trits: Trit[]): Trit {
  if (trits.length === 0) return 0;
  const sum = trits.reduce((s: number, t: number) => s + t, 0);
  return sum > 0 ? 1 : sum < 0 ? -1 : 0;
}
