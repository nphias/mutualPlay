import { HolochainService } from './holochain.service';
import { Injectable } from '@angular/core';
import { ProfilesStore } from '../stores/profiles.store';
import { serializeHash } from '../utils/utils';
import { environment } from '@environment';

export type Dictionary<T> = { [key: string]: T };

export interface Profile {
  nickname: string;
  fields: Dictionary<string>;
}

export interface AgentProfile {
  agent_pub_key: string;
  profile: Profile;
}

@Injectable({
  providedIn: "root"
})
export class ProfilesService {
  public zomeName = 'profiles'
  private agent_pub_key: string = "DEFAULT_KEY"

  constructor(private hcs:HolochainService, private pstore:ProfilesStore) { 
    this.agent_pub_key = pstore.agentPubKey
   }

  //get agent_pub_key(){return serializeHash(this.hcs.agentKeyByteArray_from_cell)};
  //get cell_holoHash(){return serializeHash(this.hcs.HoloHashByteArray_from_cell)}

  async getMyProfile(): Promise<AgentProfile> {
    const profile = await this.hcs.call(this.zomeName,'get_my_profile', null);
    if (profile)
      this.pstore.storeAgentProfile(profile)
    return profile 
  }

  async getAgentProfile(agentPubKey: string): Promise<AgentProfile> {
    const profile = await this.hcs.call(this.zomeName,'get_agent_profile', agentPubKey);
    if (profile)
      this.pstore.storeAgentProfile(profile)
    return profile 
  }

  async searchProfiles(nicknamePrefix: string): Promise<AgentProfile[]> {
    const profilesFound = await this.hcs.call(this.zomeName,'search_profiles', {
      nickname_prefix: nicknamePrefix,
    })
    if (profilesFound.length > 0)
      this.pstore.storeAllProfiles(profilesFound)
    return profilesFound
  }

  async getAllProfiles(): Promise<Array<AgentProfile>> {
    let allProfiles 
    if (environment.mock)
      allProfiles = [{agent_pub_key:this.agent_pub_key, profile:{nickname:"thomas",fields:{["email"]:"loop"}}},
      {agent_pub_key:"FRIEND_KEY", profile:{nickname:"Roberto", fields:{['email']:"doopo"}}}]
    else{
        allProfiles = await this.hcs.call(this.zomeName,'get_all_profiles', null);
    }
    if (allProfiles.length > 0)
      this.pstore.storeAllProfiles(allProfiles)
    return allProfiles
  }

  async createProfile(profile: Profile) {
    let profileResult = { agent_pub_key: this.agent_pub_key, profile:profile};
    if (!environment.mock)
      profileResult = await this.hcs.call(this.zomeName,'create_profile', profile);
    this.pstore.storeAgentProfile(profileResult)
  }
  
}