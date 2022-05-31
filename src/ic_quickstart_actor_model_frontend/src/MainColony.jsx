import React, { useState, useEffect } from 'react'

import { ic_quickstart_actor_model } from "../../declarations/ic_quickstart_actor_model"
import { createActor } from "../../declarations/ic_quickstart_actor_model"

import React from 'react'

const MainColony = () => {

    const [colonyInfo, setColonyInfo] = useState({});
    const [expeditions, setExpeditions] = useState([]);
    const [remoteColonies, setRemoteColonies] = useState([]);

    useEffect(() => {
        const interval = setInterval(() => {
            const fetchMetrics = async () => {
                const data = await ic_quickstart_actor_model.getColonyInfo();
                setColonyInfo(data);

                const exp = await ic_quickstart_actor_model.getExpeditions();
                setExpeditions(exp)

                const rem = await ic_quickstart_actor_model.getRemoteColonies();
                setRemoteColonies(rem)

                console.log(exp)
            }
            fetchMetrics();
        }, 1000);
        return () => clearInterval(interval);
    }, []);

    return (
        <>
            <div>MainColony</div>

            {
                colonyInfo.hasOwnProperty('generation') &&
                <>
                    <div> Colony id: {colonyInfo.canister_id.toString()}</div>
                    <div> Generation: {colonyInfo.generation.toString()}</div>


                    <div> Player count: {colonyInfo.player_count.toString()}</div>
                    <div> Expeditions count: {colonyInfo.expeditions_count.toString()}</div>

                    <div>Rewards per second: {colonyInfo.rewards_per_second.map(r => <> {Object.keys(r[0]).toString()} : {r[1].toString()}</>)} </div>
                    <div>Coffers: {colonyInfo.coffers.contents.map(r => <> {Object.keys(r[0]).toString()} : {r[1].toString()}</>)} </div>

                </>
            }

            <div>Expeditions:</div>

            <div className="columns is-multiline">
                {expeditions.map(exp => <Expedition expedition={exp} />)}
            </div>

            <div>Remote Colonies:</div>
            <div>{remoteColonies.map(r => <RemoteColony canisterId={r.toString()} />)}</div>
        </>
    )
}

export default MainColony

import React from 'react'

export const Expedition = ({ expedition }) => {
    // console.log(expedition[1])
    // const exp_id = expedition[0];
    const exp = expedition[1];

    const [expeditionNext, setExpeditionNext] = useState("")

    const handleExpeditionNext = async (e) => {
        e.preventDefault();

        const res = await ic_quickstart_actor_model.expeditionNext(expedition[0])
        setExpeditionNext(res)

        console.log(res)
        return true;
    }


    console.log(exp)

    return (
        <div className="column">
            <div className="box has-background-info">
                <div>Expedition ID: {exp.id.toString()}</div>
                <div>Step: {Object.keys(exp.step).toString()}</div>
                <div>Proposed At: {exp.proposed_at.toString()}</div>
                <div>Members: {exp.members.map(member => member.toString().substring(0, 5) + '...')} </div>
                <div>Resources Pool: {exp.resources_pool.contents.map(r => <> {Object.keys(r[0]).toString()} : {r[1].toString()}</>)} </div>

                <div>
                    <form onSubmit={handleExpeditionNext}>
                        <button id="clickMeBtn" type="submit">Next</button>

                    </form>
                    {
                        expeditionNext.hasOwnProperty('Err') &&
                        <div className="has-text-danger">
                            {expeditionNext.Err.toString()}
                        </div>
                    }
                </div>
            </div>


        </div>
    )
}

import React from 'react'

export const RemoteColony = ({ canisterId }) => {

    const remote_actor = createActor(canisterId);

    const [colonyInfo, setColonyInfo] = useState({});
    const [expeditions, setExpeditions] = useState([]);
    const [remoteColonies, setRemoteColonies] = useState([]);

    useEffect(() => {
        const interval = setInterval(() => {
            const fetchMetrics = async () => {
                const data = await remote_actor.getColonyInfo();
                setColonyInfo(data);

                const exp = await ic_quickstart_actor_model.getExpeditions();
                setExpeditions(exp)

                const rem = await ic_quickstart_actor_model.getRemoteColonies();
                setRemoteColonies(rem)

                console.log(exp)
            }
            fetchMetrics();
        }, 1000);
        return () => clearInterval(interval);
    }, []);

    return (
        <div>
            <div className="box has-background-warning">
                {
                    colonyInfo.hasOwnProperty('generation') &&
                    <>
                        <div> Colony id: {colonyInfo.canister_id.toString()}</div>
                        <div> Generation: {colonyInfo.generation.toString()}</div>


                        <div> Player count: {colonyInfo.player_count.toString()}</div>
                        <div> Expeditions count: {colonyInfo.expeditions_count.toString()}</div>

                        <div>Rewards per second: {colonyInfo.rewards_per_second.map(r => <> {Object.keys(r[0]).toString()} : {r[1].toString()}</>)} </div>
                        <div>Coffers: {colonyInfo.coffers.contents.map(r => <> {Object.keys(r[0]).toString()} : {r[1].toString()}</>)} </div>

                    </>
                }
            </div>
        </div >

    )
}
