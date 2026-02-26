export interface SkillContext {
  runtime: string;
  input: string;
}

export interface SkillDefinition {
  name: string;
  version: string;
  runtimes: string[];
  tools: string[];
  execute: (context: SkillContext) => Promise<unknown>;
  adapters?: Record<string, unknown>;
}

export function defineSkill(definition: SkillDefinition): SkillDefinition {
  return definition;
}
