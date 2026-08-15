import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

interface Finding {
  package: string;
  advisory: string;
  severity: 'critical' | 'high' | 'moderate';
  productionReachable: boolean;
  disposition: string;
  residualRisk: string;
  reviewTrigger: string;
}

interface RiskRegister {
  schemaVersion: 1;
  auditTool: string;
  summary: { critical: number; high: number; moderate: number; total: number };
  directProductionDependencies: string[];
  findings: Finding[];
}

export function verifyDependencyRiskRegister(
  registerPath = resolve(process.cwd(), 'operations/dependency-risk-register.json'),
  packagePath = resolve(process.cwd(), 'package.json'),
): { findings: number; directProductionHigh: number } {
  const register = readJson(registerPath, 'dependency risk register') as RiskRegister;
  const packageJson = readJson(packagePath, 'package.json') as { dependencies?: Record<string, string> };
  if (register.schemaVersion !== 1 || register.auditTool !== 'npm 10.9.8'
    || register.summary.critical !== 0 || register.summary.total !== 26
    || !Array.isArray(register.findings) || register.findings.length !== 26) {
    throw new Error('dependency risk register schema or audited baseline is invalid');
  }
  const productionDependencies = Object.keys(packageJson.dependencies ?? {}).sort();
  const baselineDependencies = [...register.directProductionDependencies].sort();
  if (productionDependencies.join('\n') !== baselineDependencies.join('\n')) {
    throw new Error('production dependency set changed without risk-register review');
  }

  const seen = new Set<string>();
  let directProductionHigh = 0;
  for (const finding of register.findings) {
    if (!finding.package || !finding.advisory || !finding.disposition || !finding.residualRisk || !finding.reviewTrigger) {
      throw new Error('dependency risk register finding is incomplete');
    }
    if (seen.has(finding.package)) {
      throw new Error('dependency risk register contains duplicate packages');
    }
    seen.add(finding.package);
    if (finding.severity === 'critical') {
      throw new Error('critical dependency vulnerability requires remediation before public pilot');
    }
    if (finding.severity === 'high' && finding.productionReachable
      && productionDependencies.includes(finding.package)) {
      directProductionHigh += 1;
    }
  }
  if (directProductionHigh > 0) {
    throw new Error('direct production high vulnerability requires explicit remediation');
  }
  return { findings: register.findings.length, directProductionHigh };
}

function readJson(path: string, label: string): unknown {
  try {
    return JSON.parse(readFileSync(path, 'utf8'));
  } catch {
    throw new Error(`${label} must be readable valid JSON`);
  }
}

if (process.argv[1]?.endsWith('dependency-risk-verify.ts')) {
  try {
    process.stdout.write(`${JSON.stringify(verifyDependencyRiskRegister())}\n`);
  } catch (error) {
    process.stderr.write(`Dependency risk verification failed: ${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  }
}
