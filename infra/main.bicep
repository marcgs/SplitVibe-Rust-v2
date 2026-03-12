@description('Base name for all resources')
param baseName string = 'splitvibe'

@description('Azure region for all resources')
param location string = resourceGroup().location

@description('PostgreSQL administrator login')
param dbAdminLogin string = 'splitvibeadmin'

@secure()
@description('PostgreSQL administrator password')
param dbAdminPassword string

@secure()
@description('Session secret for cookie signing (>= 64 bytes)')
param sessionSecret string

@secure()
@description('Google OAuth client ID')
param googleClientId string

@secure()
@description('Google OAuth client secret')
param googleClientSecret string

@description('Container image to deploy (e.g. myregistry.azurecr.io/splitvibe:latest)')
param containerImage string = '${baseName}acr.azurecr.io/splitvibe:latest'

// ── Variables ──────────────────────────────────────────────────────────
var uniqueSuffix = uniqueString(resourceGroup().id, baseName)
var vnetName = '${baseName}-vnet'
var logAnalyticsName = '${baseName}-logs-${uniqueSuffix}'
var acrName = '${baseName}acr${uniqueSuffix}'
var kvName = '${baseName}-kv-${uniqueSuffix}'
var storageName = '${baseName}st${uniqueSuffix}'
var pgName = '${baseName}-pg-${uniqueSuffix}'
var caEnvName = '${baseName}-env-${uniqueSuffix}'
var caName = '${baseName}-app'

// ── Virtual Network ────────────────────────────────────────────────────
resource vnet 'Microsoft.Network/virtualNetworks@2023-05-01' = {
  name: vnetName
  location: location
  properties: {
    addressSpace: {
      addressPrefixes: ['10.0.0.0/16']
    }
    subnets: [
      {
        name: 'postgres-subnet'
        properties: {
          addressPrefix: '10.0.1.0/24'
          delegations: [
            {
              name: 'postgres-delegation'
              properties: {
                serviceName: 'Microsoft.DBforPostgreSQL/flexibleServers'
              }
            }
          ]
        }
      }
      {
        name: 'container-apps-subnet'
        properties: {
          addressPrefix: '10.0.2.0/23'
          delegations: [
            {
              name: 'container-apps-delegation'
              properties: {
                serviceName: 'Microsoft.App/environments'
              }
            }
          ]
        }
      }
    ]
  }
}

// ── Private DNS Zone for PostgreSQL ────────────────────────────────────
resource privateDnsZone 'Microsoft.Network/privateDnsZones@2020-06-01' = {
  name: '${baseName}.private.postgres.database.azure.com'
  location: 'global'
}

resource privateDnsLink 'Microsoft.Network/privateDnsZones/virtualNetworkLinks@2020-06-01' = {
  parent: privateDnsZone
  name: '${vnetName}-link'
  location: 'global'
  properties: {
    virtualNetwork: {
      id: vnet.id
    }
    registrationEnabled: false
  }
}

// ── Log Analytics Workspace ────────────────────────────────────────────
resource logAnalytics 'Microsoft.OperationalInsights/workspaces@2022-10-01' = {
  name: logAnalyticsName
  location: location
  properties: {
    sku: {
      name: 'PerGB2018'
    }
    retentionInDays: 30
  }
}

// ── Container Registry ─────────────────────────────────────────────────
resource acr 'Microsoft.ContainerRegistry/registries@2023-07-01' = {
  name: acrName
  location: location
  sku: {
    name: 'Basic'
  }
  properties: {
    adminUserEnabled: false
  }
}

// ── Key Vault ──────────────────────────────────────────────────────────
resource keyVault 'Microsoft.KeyVault/vaults@2023-07-01' = {
  name: kvName
  location: location
  properties: {
    tenantId: subscription().tenantId
    sku: {
      family: 'A'
      name: 'standard'
    }
    enableRbacAuthorization: true
  }
}

resource secretDbPassword 'Microsoft.KeyVault/vaults/secrets@2023-07-01' = {
  parent: keyVault
  name: 'db-password'
  properties: {
    value: dbAdminPassword
  }
}

resource secretSessionSecret 'Microsoft.KeyVault/vaults/secrets@2023-07-01' = {
  parent: keyVault
  name: 'session-secret'
  properties: {
    value: sessionSecret
  }
}

resource secretGoogleClientId 'Microsoft.KeyVault/vaults/secrets@2023-07-01' = {
  parent: keyVault
  name: 'google-client-id'
  properties: {
    value: googleClientId
  }
}

resource secretGoogleClientSecret 'Microsoft.KeyVault/vaults/secrets@2023-07-01' = {
  parent: keyVault
  name: 'google-client-secret'
  properties: {
    value: googleClientSecret
  }
}

// ── Storage Account ────────────────────────────────────────────────────
resource storageAccount 'Microsoft.Storage/storageAccounts@2023-01-01' = {
  name: storageName
  location: location
  kind: 'StorageV2'
  sku: {
    name: 'Standard_LRS'
  }
}

resource blobService 'Microsoft.Storage/storageAccounts/blobServices@2023-01-01' = {
  parent: storageAccount
  name: 'default'
}

resource attachmentsContainer 'Microsoft.Storage/storageAccounts/blobServices/containers@2023-01-01' = {
  parent: blobService
  name: 'attachments'
}

// ── PostgreSQL Flexible Server ─────────────────────────────────────────
resource postgres 'Microsoft.DBforPostgreSQL/flexibleServers@2023-03-01-preview' = {
  name: pgName
  location: location
  sku: {
    name: 'Standard_B1ms'
    tier: 'Burstable'
  }
  properties: {
    version: '16'
    administratorLogin: dbAdminLogin
    administratorLoginPassword: dbAdminPassword
    storage: {
      storageSizeGB: 32
    }
    network: {
      delegatedSubnetResourceId: vnet.properties.subnets[0].id
      privateDnsZoneArmResourceId: privateDnsZone.id
    }
    highAvailability: {
      mode: 'Disabled'
    }
  }
  dependsOn: [
    privateDnsLink
  ]
}

resource postgresDb 'Microsoft.DBforPostgreSQL/flexibleServers/databases@2023-03-01-preview' = {
  parent: postgres
  name: 'splitvibe'
}

// ── Container Apps Environment ─────────────────────────────────────────
resource containerAppsEnv 'Microsoft.App/managedEnvironments@2023-05-01' = {
  name: caEnvName
  location: location
  properties: {
    appLogsConfiguration: {
      destination: 'log-analytics'
      logAnalyticsConfiguration: {
        customerId: logAnalytics.properties.customerId
        sharedKey: logAnalytics.listKeys().primarySharedKey
      }
    }
    vnetConfiguration: {
      infrastructureSubnetId: vnet.properties.subnets[1].id
    }
  }
}

// ── Container App ──────────────────────────────────────────────────────
resource containerApp 'Microsoft.App/containerApps@2023-05-01' = {
  name: caName
  location: location
  identity: {
    type: 'SystemAssigned'
  }
  properties: {
    managedEnvironmentId: containerAppsEnv.id
    configuration: {
      ingress: {
        external: true
        targetPort: 8080
        transport: 'auto'
      }
      registries: [
        {
          server: acr.properties.loginServer
          identity: 'system'
        }
      ]
      secrets: [
        {
          name: 'db-password'
          keyVaultUrl: secretDbPassword.properties.secretUri
          identity: 'system'
        }
        {
          name: 'session-secret'
          keyVaultUrl: secretSessionSecret.properties.secretUri
          identity: 'system'
        }
        {
          name: 'google-client-id'
          keyVaultUrl: secretGoogleClientId.properties.secretUri
          identity: 'system'
        }
        {
          name: 'google-client-secret'
          keyVaultUrl: secretGoogleClientSecret.properties.secretUri
          identity: 'system'
        }
      ]
    }
    template: {
      containers: [
        {
          name: 'splitvibe'
          image: containerImage
          resources: {
            cpu: json('0.5')
            memory: '1Gi'
          }
          env: [
            {
              name: 'DATABASE_URL'
              value: 'postgres://${dbAdminLogin}:${dbAdminPassword}@${postgres.properties.fullyQualifiedDomainName}:5432/splitvibe?sslmode=require'
            }
            {
              name: 'SESSION_SECRET'
              secretRef: 'session-secret'
            }
            {
              name: 'GOOGLE_CLIENT_ID'
              secretRef: 'google-client-id'
            }
            {
              name: 'GOOGLE_CLIENT_SECRET'
              secretRef: 'google-client-secret'
            }
            {
              name: 'GOOGLE_REDIRECT_URL'
              value: 'https://${caName}.${containerAppsEnv.properties.defaultDomain}/auth/google/callback'
            }
            {
              name: 'AZURE_STORAGE_ACCOUNT_NAME'
              value: storageAccount.name
            }
            {
              name: 'AZURE_STORAGE_ACCOUNT_KEY'
              value: storageAccount.listKeys().keys[0].value
            }
            {
              name: 'AZURE_STORAGE_CONTAINER_NAME'
              value: 'attachments'
            }
            {
              name: 'AZURE_STORAGE_CONNECTION_STRING'
              value: 'DefaultEndpointsProtocol=https;AccountName=${storageAccount.name};AccountKey=${storageAccount.listKeys().keys[0].value};EndpointSuffix=core.windows.net'
            }
            {
              name: 'MOCK_AUTH_ENABLED'
              value: 'false'
            }
            {
              name: 'LEPTOS_SITE_ADDR'
              value: '0.0.0.0:8080'
            }
          ]
          probes: [
            {
              type: 'Liveness'
              httpGet: {
                path: '/api/health'
                port: 8080
              }
              periodSeconds: 10
              failureThreshold: 3
            }
            {
              type: 'Readiness'
              httpGet: {
                path: '/api/health'
                port: 8080
              }
              initialDelaySeconds: 5
              periodSeconds: 10
            }
          ]
        }
      ]
      scale: {
        minReplicas: 0
        maxReplicas: 3
        rules: [
          {
            name: 'http-scale'
            http: {
              metadata: {
                concurrentRequests: '50'
              }
            }
          }
        ]
      }
    }
  }
}

// ── Role Assignments ───────────────────────────────────────────────────

// Container App → ACR Pull
resource acrPullRole 'Microsoft.Authorization/roleAssignments@2022-04-01' = {
  name: guid(acr.id, containerApp.id, 'acrpull')
  scope: acr
  properties: {
    roleDefinitionId: subscriptionResourceId('Microsoft.Authorization/roleDefinitions', '7f951dda-4ed3-4680-a7ca-43fe172d538d')
    principalId: containerApp.identity.principalId
    principalType: 'ServicePrincipal'
  }
}

// Container App → Key Vault Secrets User
resource kvSecretsRole 'Microsoft.Authorization/roleAssignments@2022-04-01' = {
  name: guid(keyVault.id, containerApp.id, 'kvsecrets')
  scope: keyVault
  properties: {
    roleDefinitionId: subscriptionResourceId('Microsoft.Authorization/roleDefinitions', '4633458b-17de-408a-b874-0445c86b69e6')
    principalId: containerApp.identity.principalId
    principalType: 'ServicePrincipal'
  }
}

// ── Outputs ────────────────────────────────────────────────────────────
output containerAppFqdn string = containerApp.properties.configuration.ingress.fqdn
output acrLoginServer string = acr.properties.loginServer
output postgresHost string = postgres.properties.fullyQualifiedDomainName
