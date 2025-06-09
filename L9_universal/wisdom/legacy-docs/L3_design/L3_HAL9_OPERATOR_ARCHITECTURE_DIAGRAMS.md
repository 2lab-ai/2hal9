# HAL9-Operator Architecture Diagrams

## System Overview

```mermaid
graph TB
    subgraph "Users"
        U1[User 1]
        U2[User 2]
        UN[User N]
    end
    
    subgraph "Telegram"
        TB[Telegram Bot API]
        TS[Telegram Stars]
    end
    
    subgraph "HAL9-Operator Core"
        GW[API Gateway]
        UM[User Manager]
        NP[Neuron Pool]
        SM[Session Manager]
    end
    
    subgraph "AI Neurons"
        N1[Neuron 1<br/>User 1]
        N2[Neuron 2<br/>User 2]
        NN[Neuron N<br/>User N]
    end
    
    subgraph "Browser Automation"
        BP[Browser Pool]
        BA1[Browser 1]
        BA2[Browser 2]
        BAN[Browser N]
    end
    
    subgraph "External Services"
        AMZ[Amazon]
        TRIP[Trip.com]
        TON[TON Blockchain]
    end
    
    U1 & U2 & UN --> TB
    TB --> GW
    TS --> GW
    GW --> UM
    UM --> NP
    NP --> N1 & N2 & NN
    N1 & N2 & NN --> BP
    BP --> BA1 & BA2 & BAN
    BA1 & BA2 & BAN --> AMZ & TRIP
    GW --> TON
```

## Data Flow Diagram

```mermaid
sequenceDiagram
    participant User
    participant Telegram
    participant HAL9
    participant Neuron
    participant Browser
    participant Amazon
    participant TON
    
    User->>Telegram: "Buy iPhone case under $20"
    Telegram->>HAL9: Message webhook
    HAL9->>Neuron: Route to user's neuron
    Neuron->>Browser: Create session
    Browser->>Amazon: Navigate & search
    Amazon->>Browser: Search results
    Browser->>Neuron: Extract products
    Neuron->>User: Show top 3 options
    User->>Neuron: "Buy the first one"
    Neuron->>Browser: Add to cart
    Browser->>Amazon: Checkout
    Neuron->>TON: Deduct USDT
    TON->>Neuron: Payment confirmed
    Amazon->>Browser: Order confirmed
    Browser->>Neuron: Order details
    Neuron->>User: "Order placed! Tracking: #123"
```

## Component Architecture

```mermaid
graph LR
    subgraph "MCP Tools Layer"
        MT1[browser_session]
        MT2[browser_navigate]
        MT3[browser_interact]
        MT4[browser_extract]
        MT5[amazon_search]
        MT6[amazon_checkout]
        MT7[ton_wallet]
        MT8[telegram_send]
    end
    
    subgraph "Core Services"
        BS[Browser Service]
        PS[Payment Service]
        NS[Notification Service]
        SS[Security Service]
    end
    
    subgraph "Data Layer"
        PG[(PostgreSQL)]
        RD[(Redis)]
        FS[(File Storage)]
    end
    
    MT1 & MT2 & MT3 & MT4 --> BS
    MT5 & MT6 --> PS
    MT7 --> PS
    MT8 --> NS
    
    BS & PS & NS --> SS
    SS --> PG & RD & FS
```

## Security Architecture

```mermaid
graph TB
    subgraph "User Layer"
        U[User]
        MFA[2FA/MFA]
    end
    
    subgraph "Application Layer"
        API[API Gateway]
        AUTH[Auth Service]
        VAULT[Credential Vault]
    end
    
    subgraph "Security Layer"
        FW[Firewall]
        WAF[Web App Firewall]
        IDS[Intrusion Detection]
        RISK[Risk Analysis]
    end
    
    subgraph "Data Layer"
        ENC[Encryption at Rest]
        KMS[Key Management]
        AUDIT[Audit Logs]
    end
    
    U --> MFA --> API
    API --> AUTH
    AUTH --> VAULT
    API --> FW --> WAF
    WAF --> IDS
    IDS --> RISK
    VAULT --> ENC
    ENC --> KMS
    RISK --> AUDIT
```

## Deployment Architecture

```mermaid
graph TB
    subgraph "Load Balancer"
        LB[NGINX/ALB]
    end
    
    subgraph "Kubernetes Cluster"
        subgraph "API Pods"
            API1[hal9-api-1]
            API2[hal9-api-2]
            APIN[hal9-api-n]
        end
        
        subgraph "Neuron Pods"
            NP1[neuron-pod-1]
            NP2[neuron-pod-2]
            NPN[neuron-pod-n]
        end
        
        subgraph "Browser Pods"
            BP1[browser-pod-1]
            BP2[browser-pod-2]
            BPN[browser-pod-n]
        end
    end
    
    subgraph "Data Tier"
        PG[(PostgreSQL<br/>Primary)]
        PGR[(PostgreSQL<br/>Replica)]
        REDIS[(Redis Cluster)]
        S3[(S3/Minio)]
    end
    
    subgraph "Monitoring"
        PROM[Prometheus]
        GRAF[Grafana]
        ELK[ELK Stack]
    end
    
    LB --> API1 & API2 & APIN
    API1 & API2 & APIN --> NP1 & NP2 & NPN
    NP1 & NP2 & NPN --> BP1 & BP2 & BPN
    API1 & API2 & APIN --> PG & REDIS
    PG --> PGR
    API1 & API2 & APIN --> S3
    API1 & API2 & APIN --> PROM
    PROM --> GRAF
    API1 & API2 & APIN --> ELK
```

## Transaction Flow

```mermaid
stateDiagram-v2
    [*] --> UserRequest: User sends command
    UserRequest --> ValidateUser: Check subscription
    ValidateUser --> AllocateNeuron: Valid subscription
    ValidateUser --> [*]: Invalid/Expired
    
    AllocateNeuron --> ParseIntent: Assign neuron
    ParseIntent --> BrowseProduct: Shopping intent
    ParseIntent --> CheckBalance: Balance check
    ParseIntent --> ScheduleTask: Reminder setup
    
    BrowseProduct --> CreateSession: Initialize browser
    CreateSession --> NavigateSite: Go to e-commerce site
    NavigateSite --> SearchProduct: Execute search
    SearchProduct --> ExtractResults: Parse results
    ExtractResults --> PresentOptions: Show to user
    PresentOptions --> UserConfirm: Await confirmation
    UserConfirm --> ProcessPayment: User approves
    UserConfirm --> [*]: User cancels
    
    ProcessPayment --> DeductBalance: USDT payment
    ProcessPayment --> UseCard: Card payment
    DeductBalance --> CompleteOrder: Balance sufficient
    DeductBalance --> [*]: Insufficient funds
    UseCard --> CompleteOrder: Card authorized
    CompleteOrder --> TrackOrder: Order placed
    TrackOrder --> [*]: Send confirmation
```

## Scaling Strategy

```mermaid
graph LR
    subgraph "Phase 1: MVP"
        U1[100 Users]
        N1[10 Neurons]
        B1[5 Browsers]
    end
    
    subgraph "Phase 2: Growth"
        U2[1K Users]
        N2[100 Neurons]
        B2[50 Browsers]
    end
    
    subgraph "Phase 3: Scale"
        U3[10K Users]
        N3[1K Neurons]
        B3[500 Browsers]
    end
    
    subgraph "Phase 4: Enterprise"
        U4[100K Users]
        N4[10K Neurons]
        B4[5K Browsers]
    end
    
    U1 --> U2 --> U3 --> U4
    N1 --> N2 --> N3 --> N4
    B1 --> B2 --> B3 --> B4
```

## Integration Points

```mermaid
graph TB
    subgraph "HAL9 Core"
        CORE[HAL9-Operator]
    end
    
    subgraph "Messaging"
        TG[Telegram API]
        WH[WhatsApp Business]
        DC[Discord API]
    end
    
    subgraph "E-commerce"
        AMZ[Amazon API/Scraping]
        TRIP[Trip.com API]
        EBAY[eBay API]
        ALI[AliExpress API]
    end
    
    subgraph "Blockchain"
        TON[TON Network]
        ETH[Ethereum]
        BSC[Binance Smart Chain]
    end
    
    subgraph "Payment"
        STRIPE[Stripe]
        PP[PayPal]
        CB[Coinbase Commerce]
    end
    
    CORE --> TG & WH & DC
    CORE --> AMZ & TRIP & EBAY & ALI
    CORE --> TON & ETH & BSC
    CORE --> STRIPE & PP & CB
```

These diagrams provide a comprehensive visual overview of the HAL9-Operator architecture, showing how all components interact to deliver autonomous shopping and financial management capabilities through Telegram.