#!/usr/bin/env node
import 'source-map-support/register'
import * as cdk from 'aws-cdk-lib'
import { PawStack } from '../lib/paw_stack'

const app = new cdk.App()
new PawStack(app, 'PawStack', {})
