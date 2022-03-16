# How blockchain can ease electromobility

Electromobility is essential element in energy transition and sustainable city, and it is an unmissable element in transport sector evolution. Electromobility allows the synergy between ecological, social, and technical values. 
Europe have a strong will to support and help the electric car market development. Thanks to European Government actions it is now commonly accepted that only electric vehicles will be sold by vendors in few years.
But passing from gas vehicles to electric vehicles is not as easy as it seems. Most people are not convinced by electromobility due to the difficulty to find a charging station, to the lack of transparency regarding charging session price and to the charging time.
Therefore, facilitate electromobility development will pass by optimizing infrastructures and increasing the number of public charging spots. 

At Delmonicos, we are convinced that blockchain technology can ease electromobility adoption by optimizing the charging process while enhancing security and simplifying the user experience. 

## A complex value chain carried out by a multitude of actors
As reflected by the following schema (Figure 1), most people think charging process of an electric vehicle is quite straightforward: 
You connect the charging station to the vehicle 
You start the charging process and then confirm that you are ok to use this station to charge your vehicle 
When you want to stop, you unplug the cable, the price is calculated, and a payment is executed from the client bank account to the station owner bank account



![The ideal process](./Ideal_process.png)

As we can see in the Figure 2, reality is far more complex. Multiple actors are involved during the charging process to ensure that payment is processed and completed according to the EV (electric vehicle) owner consent.  

![The reality](./real_process.png)

More than understanding the entire electric charging process, it is important to identify each actor and explain their roles: 

* **CPO** (Charge Point Operator) manages and supervises the infrastructure of charging stations, usually in public spaces. It can operate one or many charging infrastructure behalf of station owners or for its own account. CPO uses **CPMS** (Charge Point Managing System), the software developed by **CPMS development company** used to manage and supervise the charging stations.
* **eMSP** (eMobility Service Provider) is providing mobility services like giving access to charging stations to EV users. 
* **Aggregator** (Or Clearing House or hub) is a platform offering services that enable mobility operators to easily build partnerships. It develops the links between CPO and eMSP to enables electric mobility roaming.

Those three actors are the main ones, but there are other secondaries like **OEM Constructors** who builds Charging Points, **NSP** (Navigation Service Provider) who provides location information of Charge Points, **Integrators** who supports new players who want to setup their own Charge Station, ...

Moreover, some actors take several roles like a CPO who can be eMSP and develop is own CPMS.

To help all these actors to communicate together, some standards have been set up to manage the electromobility ecosystem.

## A complex value chain handled by a multitude of protocols
Every distributed system has at least one protocol that define the common language, also called standard. This language is used by each actor to communicate and exchange data and values. 

In electromobility there are mainly 2 protocols that we can define as the standards of the market (OCPP and OCPI). But there are a lot of other protocols that are actually still used.

![EVSE protocols](./evse_protocols.webp)
(Image source: [ElaadNL EV related protocol study v1.1](https://www.elaad.nl/uploads/downloads/downloads_download/EV_related_protocol_study_v1.1.pdf)))

**EVSE** (Electric Vehicle Supply Equipment) means equipment that delivers electrical energy from an electricity source to charge plug-in electric vehicle batteries (Charging Point, Chargin Station, Charging Dock, ...)
**DSO** (Distribution System Operator) operates the electric power distribution system which delivers electricity

## OCPP protocol

Protocol OCPP (Open Charge Point Protocol) allows the communication between **EVSE** and **CPMS**. 

This protocol exists in two major versions 1.6 and 2.0.1. The 1.6 is the most used version, the 2.0.1 will probably soon replace the 1.6 as it allows to handle more complexes CPMS and makes OCPP compatible with the ISO15118 features.

The common way to implement this protocol is to use a SecureWebSocket between the **EVSE** and the **CPMS** through a VPN (or HTTPS) for exchanging JSON messages.

## OCPI protocol

The OCPI protocol is used in two ways :
- in replacement of the private protocols used by the aggregators to exchange with CPO
- to exchange directly between eMSP and CPO without passing by an aggregator. It's a not very common using of the protocol, because it's difficult to implement it for this purpose

## ISO 15118 standard

Another standard is the ISO 15118 which aims to improve the electromobility ecosystem. Its objective is to simplify EV drivers’ life, to allow energy optimization, to reduce costs and to strengthen (cyber) security. One part of ISO 15118 is named “Plug&Charge”, it allows the charging station to identify the user's service contract and automatically charge the vehicle only by plugin the EV to the station.

With Delmonicos, we are creating a new generation of software that will make any equipped charging station natively compatible with the submentionned standards and open to any kind of payment. 

## Delmonicos inside
Delmonicos wants to make the world of e-mobility easier and will allow each electric vehicle driver to be able to charge at any smart charging station without any fear or doubt about the cost they will be charged.

![Delmonicos inside](./delmonicos_inside.png)
To do this, Delmonicos aims to equip each charge point with its innovative and universal software solution "Delmonicos inside" which will be composed by an operating system and a monitoring platform. 
In addition, each user of the solution will be able to proceed payment either with his usual mobility application and his payment application to which the Delmonicos API has been integrated or he can simply use the Delmonicos application.

This solution will provide electric vehicle drivers an experience close to what a "Tesla" owner lives on a "Plug & Charge" Supercharger. He simply has to connect his vehicle to the Supercharger to start charging. No identification problems or pricing surprises.

To achieve this challenge, Delmonicos is working on Blockchain technology based on the principle of trusted third parties and provides to our future users a new generation, simple and interconnected solution.

## Why blockchain for electromobility?  

As we have seen the main problem is related to the payment of the charging session. 
We are developing and deploying an infrastructure dedicated to the supervision of charging station, strongly coupled with a micro-payment infrastructure. The main requirements of the solution are the following :

* It must be very **secure**. Especially, only allowed people and computers should be able to interact with charging stations. Payments must fulfill latests security constraints especially regarding DSP2.
* It must be as **cheap** as possible to operate. The challenge is to meet the security requirements without having additonal costs or having to rely on external third-party.
* It must be **easy to deploy**.
* It must be **easily scalable**. Other parties should be able to join the networks without compromising the security of the infrastructure.

Taking into account the above-mentioned constraints, we have chosen to base our solution on blockchain technology, for the following reasons :

* It is secure by design and quasi impossible to hack if carefully used and implemented.
* It allows strong authentication through the use of wallets
* It allows secured delivery versus payment. Each payment can be related to a dedicated delivery of service and the proofs of the delivery can be stored in case of dispute.
* The trustless nature of the technology allows adding new network participants without compromising the security of other participants.
* The deployment can be easy and secured if each charging station is also a network node.

## Why Polkadot ?
Finally, we chose Substrate/Polkadot because :

* By having nodes with a dedicated purpose, we will be able to optimise the footprint of the node in order to deploy it to charging stations, that have limited available resources
* We can choose the most suitable consensus mechanism and especially combine POA on the parachain with PoS on the relay chain.
* We have the flexibility to chose between multiple target topologies. E.g. one parachain for all energy providers on the platform, or one parachain by energy provider.
The composability of the FRAME architecture allows us to reuse existing pallets (e.g. identity, membership, contracts, ...) while being able to add our own specific pallets.
Since we have connections to multiple interfaces (payments, charging power, mobile apps) the offchain features of Substrate will alllow us to handle the connection with external sources in both directions in a secure manner.
