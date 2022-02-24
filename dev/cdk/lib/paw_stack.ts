import { Stack, StackProps } from 'aws-cdk-lib'
import { Construct } from 'constructs'
import { PawLambda } from '../lib/lambda'

export class PawStack extends Stack {
    constructor(scope: Construct, id: string, props?: StackProps) {
        super(scope, id, props)

        new PawLambda(this, 'PawLambda')
    }
}
