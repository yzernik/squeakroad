import React from 'react'
import { withRouter , Link } from 'react-router-dom'
import './style.scss'
import Squeak from '../../features/squeaks/Squeak'


const SqueakPage = (props) => {

    return(
        <>
        <Squeak id={props.match.params.id} />
        </>
    )
}

export default withRouter(SqueakPage)
