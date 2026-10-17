import { loadStore } from "./mockStore";
import { handleCaseOps } from "./mockCaseOps";
import { handleFeatureOps } from "./mockFeatures";
import { handleQuitamOps } from "./mockQuitam";

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

  const quitamRes = handleQuitamOps<T>(cmd, args, store);
  if (quitamRes !== null) {
    return quitamRes;
  }

  throw `Unknown command in browser preview: ${cmd}`;
}
