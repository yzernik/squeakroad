import React, {useContext} from 'react'
import './style.scss'

const Alert = (props) => {

    // TODO: Get these values from a state or slice.
    const top = '-100px';
    const msg = '';

    return(
        <div style={{top: top}} className="alert-wrapper">
            <div className="alert-content">
                {msg}
            </div>
        </div>
    )
}

export default Alert
