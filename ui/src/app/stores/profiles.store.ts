import {Injectable} from "@angular/core"
import { Hashed, serializeHash } from '../utils/utils';
import {
  observable,
  makeObservable,
  action,
  runInAction,
  computed,
} from 'mobx';
import { Dictionary, Profile, AgentProfile } from '../services/profiles.service';
import { environment } from "@environment";

@Injectable({
  providedIn: "root"
})
export class ProfilesStore {
  @observable
  private profiles: Dictionary<Profile> = {}//= {['DEFAULT_KEY']:{nickname:"Thomas", fields:{['email']:"looop"}}};
  @observable
  private my_agent_pub_key!:string //= "DEFAULT_KEY"

  constructor() {
    makeObservable(this);
  }

  profileOf(agentPubKey: string): Profile {
    return this.profiles[agentPubKey];
  }

  get agentPubKey():string {
    return this.my_agent_pub_key    
    //   return serializeHash(this.profilesService.cell_agentKeyByteArray);
  }

  set agentPubKey(agent_pub_key:string){
    this.my_agent_pub_key = agent_pub_key
  }

  @computed
  get MyProfile(): Profile  { //| undefined
    return this.profiles[this.agentPubKey];
  }

  @computed
  get knownProfiles(): Array<AgentProfile> {
    return Object.entries(this.profiles).map(([agent_pub_key, profile]) => ({
      agent_pub_key,
      profile,
    }));
  }

  @action
  public async storeAllProfiles(allProfiles:Array<AgentProfile>) {
    //const allProfiles = await this.profilesService.getAllProfiles();
 // if( environment.mock){
 //   this.profiles["Friend_KEY"] = {nickname:"Roberto", fields:{['email']:"doopo"}}
 // }
 //console.log(allProfiles.length)
    runInAction(() => {
      for (const agentProfile of allProfiles) {
        this.profiles[agentProfile.agent_pub_key] = agentProfile.profile;
      }
    });
  }

  @action
  public async storeAgentProfile(profile:AgentProfile) {
    //const profile = await this.profilesService.getAgentProfile(agentPubKey);

    if (profile) {
      runInAction(() => {
        this.profiles[profile.agent_pub_key] = profile.profile;
      });
    }
  }

}