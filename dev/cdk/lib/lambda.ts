import { Construct } from 'constructs'
import * as lambda from 'aws-cdk-lib/aws-lambda'
import { Bucket } from 'aws-cdk-lib/aws-s3'

export class PawLambda extends Construct {
    readonly pawLambdaHandler: lambda.Function

    constructor(scope: Construct, id: string) {
        super(scope, id)

        const mainFunction = `def lambda_handler(input, context):
    status_code = 200
    bode = input["body"]

    if (bode == "erro"):
        status_code = 404

    return {
        'statusCode': status_code,
        'body': bode
    }`

        const bucket = new Bucket(this, 'PawLambdaBucket')

        this.pawLambdaHandler = new lambda.Function(this, 'PawLambdaHandler', {
            functionName: 'PawLambda',
            runtime: lambda.Runtime.PYTHON_3_9,
            code: lambda.Code.fromInline(mainFunction),
            handler: 'index.lambda_handler',
            environment: {
                BUCKET: bucket.bucketName,
            },
        })

        bucket.grantReadWrite(this.pawLambdaHandler)
    }
}
