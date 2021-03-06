import { Component, OnInit } from "@angular/core";
import { Observable, Subscription } from 'rxjs';
import { map } from 'rxjs/operators';
import { Router } from "@angular/router";
import { AllAgentsGQL,Agent } from 'src/app/graphql/profile/queries/all-agents-gql';
import { CreateOfferGQL } from 'src/app/graphql/transactor/queries/offer-mutations-gql';
import { FormBuilder, FormGroup, Validators, FormArray } from '@angular/forms';
import { MyOffersGQL } from 'src/app/graphql/transactor/queries/myoffers-gql';

interface offerRow
{
  id:string,
  username:string,
  amount:number
} 

@Component({
  selector: "app-userlist",
  templateUrl: "./userlist.component.html",
  styleUrls: ["./userlist.component.css"]
})
export class UserListComponent implements OnInit {
  agentlist: Observable<Agent[]>;
  errorMessage:string
  userForm: FormGroup
  agentSubscription: Subscription

  constructor(
    private agents: AllAgentsGQL, 
    private offer: CreateOfferGQL,
    private offers:MyOffersGQL,
    private router: Router,
    private fb: FormBuilder
    ) { 
  }

  ngOnInit() {
    this.userForm = this.fb.group({
      Rows : this.fb.array([])
    });
    try {
      this.agentlist = this.agents.watch().valueChanges.pipe(map(result=>{
        if (!result.errors)
          return result.data.allAgents.map(agent => <Agent>{id:agent.id, username:agent.username})
        this.errorMessage = result.errors[0].message
        return null
      }))
    } catch(exception){
        this.errorMessage = exception
    }
    this.agentSubscription = this.agentlist.subscribe(agents => { this.populateForm(agents)})
  }

  ngOnDestroy(){
    this.agentSubscription.unsubscribe()
  }

  get formArr() {
    return this.userForm.get("Rows") as FormArray;
  }

  populateForm(agentlist: Agent[]){
    for (let i = 0; i < agentlist.length; i++ ) {
      if (agentlist[i].id != sessionStorage.getItem("userhash")){
        this.formArr.push(
          this.fb.group({
            id: this.fb.control(agentlist[i].id),
            username: this.fb.control(agentlist[i].username),
            amount: this.fb.control(0)
          })
        )
      }
    }
  }

  async createOffer(data:offerRow){
    console.log(data)
    try {
      await this.offer.mutate({creditorId:data.id,amount:data.amount},{refetchQueries: [{query: this.offers.document}]})
      .toPromise()//.then(result => {
      //console.log(result)
      this.router.navigate(["offers"]);
    //}).catch(ex=>{this.errorMessage = ex})
    } catch(exception){
      console.error(exception)
      if ((exception as string).includes("Timeout"))
        exception = "No reponse, the user is probably offline:"+exception
      this.errorMessage = exception
       // console.log(exception)
       // this.errorMessage = exception
    }
  }
  

}
