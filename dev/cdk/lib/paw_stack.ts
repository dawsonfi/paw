import { Stack, StackProps } from 'aws-cdk-lib'
import { Construct } from 'constructs'
import { PawLambda } from '../lib/lambda'
import { PawStateMachine } from '../lib/step_functions'

export class PawStack extends Stack {
    constructor(scope: Construct, id: string, props?: StackProps) {
        super(scope, id, props)

        const pawLambda = new PawLambda(this, 'PawLambda')
        new PawStateMachine(this, 'PawStateMachine', pawLambda.pawLambdaHandler)
    }
}
