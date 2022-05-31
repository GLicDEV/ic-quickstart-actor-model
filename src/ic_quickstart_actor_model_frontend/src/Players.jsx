import React, { useEffect, useState } from 'react'

import { Secp256k1KeyIdentity } from "@dfinity/identity"
import { createActor } from "../../declarations/ic_quickstart_actor_model"
const canisterId = process.env.IC_QUICKSTART_ACTOR_MODEL_CANISTER_ID;

const Players = () => {
    return (
        <>
            <div className="columns is-multiline">
                <Player lastByte="1" />
                <Player lastByte="2" />
                <Player lastByte="3" />
            </div>
        </>
    )
}

export default Players

import React from 'react'

export const Player = ({ lastByte }) => {

    const rawBuffer = new Uint8Array([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, lastByte]);
    const test_id = Secp256k1KeyIdentity.generate(rawBuffer);
    const [greet, setGreet] = useState(false);
    const [isPlaying, setIsPlaying] = useState(false);
    const [inventory, setInventory] = useState({});
    const [claimed, setClaimed] = useState(false);
    const [playerStatus, setPlayerStatus] = useState({})

    const test = createActor(canisterId, {
        agentOptions: {
            identity: test_id,
        }
    });

    const handleSubmit = async (e) => {
        e.preventDefault();

        const res = await test.addPlayerToWorld()
        // console.log(res)

        setIsPlaying(true)

        return false;

    }

    const handleStartWork = async (e) => {
        e.preventDefault();

        const res = await test.startWork();

        // console.log(res)
        return true;
    }

    const handleStopWork = async (e) => {
        e.preventDefault();

        const res = await test.stopWork()
        setClaimed(!claimed)

        console.log(res)
        return true;
    }

    const handleStartExpedition = async (e) => {
        e.preventDefault();

        const res = await test.startExpedition();
        setClaimed(!claimed)

        console.log(res)
        return true;
    }

    const handleJoinExpedition = async (e) => {
        e.preventDefault();

        const res = await test.joinExpedition(0);
        setClaimed(!claimed)

        console.log(res)
        return true;
    }

    useEffect(async () => {
        if (!greet) {
            const data = await test.isPlayerHere();
            setGreet(data)
        }
        const inv = await test.getPlayerInventory();
        setInventory(inv);

        console.log(inv)

    }, [isPlaying, claimed]);

    // // Use for player status
    // useEffect(() => {
    //     const interval = setInterval(() => {
    //         const fetchMetrics = async () => {
    //             const data = await test.isPlayerHere();
    //             setPlayerStatus(data);
    //             // console.log(data)
    //         }
    //         fetchMetrics();
    //     }, 1000);
    //     return () => clearInterval(interval);
    // }, []);

    return (
        <div className="column">
            <div className="box has-background-success">

                <div>
                    ID: {lastByte}
                </div>

                {
                    !greet &&
                    <>
                        <div className="block">

                            Welcome, join the main world to start the game
                            <form onSubmit={handleSubmit}>
                                <button id="clickMeBtn" type="submit">Join world</button>

                            </form>
                        </div>
                    </>
                }
                {
                    greet &&
                    <>
                        <div className="block">

                            {
                                inventory.contents?.length == 0 &&
                                <>
                                    <div>
                                        Your inventory is empty. Start work to gain some resources.
                                    </div>
                                </>

                            }
                            {
                                inventory.contents?.length > 0 &&
                                <>
                                    <div>Inventory: {inventory.contents.map(r => <> {Object.keys(r[0]).toString()} : {r[1].toString()}</>)} </div>
                                </>
                            }
                        </div>

                        {/* <div className="block">

                            Player Status: {playerStatus.toString()}

                        </div> */}

                        <div className="columns is-multiline">
                            <div><form onSubmit={handleStartWork}>
                                <button id="clickMeBtn" type="submit">Start Work</button>

                            </form></div>
                            <div>
                                <form onSubmit={handleStopWork}>
                                    <button id="clickMeBtn" type="submit">Claim Work</button>

                                </form>
                            </div>

                        </div>

                        <div className="block">
                            <div>
                                <form onSubmit={handleStartExpedition}>
                                    <button id="clickMeBtn" type="submit">Start expedition</button>

                                </form>
                            </div>

                            <div>
                                <form onSubmit={handleJoinExpedition}>
                                    <button id="clickMeBtn" type="submit">Join expedition ID: 0</button>

                                </form>
                            </div>

                        </div>

                    </>
                }
            </div>
        </div>
    )
}
