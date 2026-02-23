import { get, post, del } from "../web-client";
import type { AppId } from "./types";

export interface SkillApps {
  claude: boolean;
  codex: boolean;
  gemini: boolean;
  opencode: boolean;
  openclaw: boolean;
}

export interface InstalledSkill {
  id: string;
  name: string;
  description?: string;
  directory: string;
  repoOwner?: string;
  repoName?: string;
  repoBranch?: string;
  readmeUrl?: string;
  apps: SkillApps;
  installedAt: number;
}

export interface DiscoverableSkill {
  key: string;
  name: string;
  description: string;
  directory: string;
  readmeUrl?: string;
  repoOwner: string;
  repoName: string;
  repoBranch: string;
}

export interface UnmanagedSkill {
  directory: string;
  name: string;
  description?: string;
  foundIn: string[];
  path: string;
}

export interface Skill {
  key: string;
  name: string;
  description: string;
  directory: string;
  readmeUrl?: string;
  installed: boolean;
  repoOwner?: string;
  repoName?: string;
  repoBranch?: string;
}

export interface SkillRepo {
  owner: string;
  name: string;
  branch: string;
  enabled: boolean;
}

export const skillsApi = {
  async getInstalled(): Promise<InstalledSkill[]> {
    return get("/skills/installed");
  },

  async installUnified(
    skill: DiscoverableSkill,
    _currentApp: AppId,
  ): Promise<InstalledSkill> {
    return post(`/skills/${skill.key}/install`, { skill, currentApp: _currentApp });
  },

  async uninstallUnified(id: string): Promise<boolean> {
    return del(`/skills/${id}/uninstall`);
  },

  async toggleApp(id: string, app: AppId, enabled: boolean): Promise<boolean> {
    return post(`/skills/${id}/toggle`, { app, enabled });
  },

  async scanUnmanaged(): Promise<UnmanagedSkill[]> {
    return get("/skills/unmanaged");
  },

  async importFromApps(directories: string[]): Promise<InstalledSkill[]> {
    return post("/skills/import", { directories });
  },

  async discoverAvailable(): Promise<DiscoverableSkill[]> {
    return get("/skills/discover");
  },

  async getAll(app: AppId = "claude"): Promise<Skill[]> {
    return get(`/skills?app=${app}`);
  },

  async install(directory: string, app: AppId = "claude"): Promise<boolean> {
    return post("/skills/install", { directory, app });
  },

  async uninstall(directory: string, app: AppId = "claude"): Promise<boolean> {
    return del(
      `/skills/uninstall?directory=${encodeURIComponent(directory)}&app=${app}`,
    );
  },

  async getRepos(): Promise<SkillRepo[]> {
    return get("/skills/repos");
  },

  async addRepo(repo: SkillRepo): Promise<boolean> {
    return post("/skills/repos", repo);
  },

  async removeRepo(owner: string, name: string): Promise<boolean> {
    return del(`/skills/repos/${owner}/${name}`);
  },

  async openZipFileDialog(): Promise<string | null> {
    console.warn("open_zip_file_dialog not available in web mode");
    return null;
  },

  async installFromZip(
    _filePath: string,
    _currentApp: AppId,
  ): Promise<InstalledSkill[]> {
    console.warn("install_skills_from_zip not available in web mode");
    return [];
  },
};
