import { loadStore } from "./mockStore";
import { handleCaseOps } from "./mockCaseOps";
import { handleFeatureOps } from "./mockFeatures";

export async function mockInvoke<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  const store = loadStore();

  const caseRes = handleCaseOps<T>(cmd, args, store);
  if (caseRes !== null) {
    return caseRes;
  }

  const featureRes = handleFeatureOps<T>(cmd, args, store);
  if (featureRes !== null) {
    return featureRes;
  }

  throw `Unknown command in browser preview: ${cmd}`;
}
