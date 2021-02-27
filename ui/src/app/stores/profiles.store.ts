import {Injectable} from "@angular/core"
import { Hashed, serializeHash } from '../utils/utils';
import {
  observable,
  makeObservable,
  action,
  runInAction,
  computed,
} from 'mobx';
import { ProfilesService, Dictionary, Profile, AgentProfile } from '../services/profiles.service';

@Injectable({
  providedIn: "root"
})
export class ProfilesStore {
  @observable
  public profiles: Dictionary<Profile> = {};

  constructor(protected profilesService: ProfilesService) {
    makeObservable(this);
  }

  profileOf(agentPubKey: string): Profile {
    return this.profiles[agentPubKey];
  }

  get myAgentPubKey() {
    return serializeHash(this.profilesService.cell_agentKeyByteArray);
  }

  @computed
  get myProfile(): Profile | undefined {
    return this.profiles[this.myAgentPubKey];
  }

  @computed
  get knownProfiles(): Array<AgentProfile> {
    return Object.entries(this.profiles).map(([agent_pub_key, profile]) => ({
      agent_pub_key,
      profile,
    }));
  }

  @action
  public async fetchAllProfiles() {
    const allProfiles = await this.profilesService.getAllProfiles();

    runInAction(() => {
      for (const agentProfile of allProfiles) {
        this.profiles[agentProfile.agent_pub_key] = agentProfile.profile;
      }
    });
  }

  @action
  public async fetchAgentProfile(agentPubKey: string) {
    const profile = await this.profilesService.getAgentProfile(agentPubKey);

    if (profile) {
      runInAction(() => {
        this.profiles[agentPubKey] = profile.profile;
      });
    }
  }

  @action
  public async fetchMyProfile() {
    const myProfile = await this.profilesService.getMyProfile();
    console.log(myProfile)
    if (myProfile) {
      runInAction(() => {
        this.profiles[this.myAgentPubKey] = myProfile.profile;
      });
    }
  }

  @action
  public async searchProfiles(nicknamePrefix: string) {
    const searchedProfiles = await this.profilesService.searchProfiles(
      nicknamePrefix
    );

    runInAction(() => {
      for (const { agent_pub_key, profile } of searchedProfiles) {
        this.profiles[agent_pub_key] = profile;
      }
    });
  }

  @action
  public async createProfile(profile: Profile) {
    await this.profilesService.createProfile(profile);

    runInAction(() => {
      this.profiles[this.myAgentPubKey] = profile;
    });
  }
}