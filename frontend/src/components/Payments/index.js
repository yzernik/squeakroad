import React, { useEffect, useState, useContext } from 'react'
import './style.scss'
import moment from 'moment'
import { withRouter, Link } from 'react-router-dom'
import { ICON_SEARCH, ICON_ARROWBACK, ICON_CLOSE } from '../../Icons'
import { getProfileImageSrcString } from '../../squeakimages/images';
import Loader from '../Loader'
import SqueakCard from '../SqueakCard'
import SentPayments from '../../features/payments/SentPayments'
import ReceivedPayments from '../../features/payments/ReceivedPayments'


const Payments = (props) => {
    const [tab, setTab] = useState('Sent Payments')
    const [styleBody, setStyleBody] = useState(false)

    return(
        <div>

        <div className="explore-wrapper">
            <div className="payments-header-wrapper">
                <div className="payments-header-content">
                    <div className="payments-header-name">
                        Payments
                    </div>
                </div>
            </div>
            <div className="profile-details-wrapper">
            <div className="profiles-options">
            </div>
            </div>
            <div>
                <div className="explore-nav-menu">
                    <div onClick={()=>setTab('Sent Payments')} className={tab === 'Sent Payments' ? `explore-nav-item activeTab` : `explore-nav-item`}>
                        Sent Payments
                    </div>
                    <div onClick={()=>setTab('Received Payments')} className={tab === 'Received Payments' ? `explore-nav-item activeTab` : `explore-nav-item`}>
                        Received Payments
                    </div>
                </div>
                {tab === 'Sent Payments' ?
                <>
                <SentPayments />
                </>

                :
                tab === 'Received Payments' ?
                  <>
                    <ReceivedPayments />
                  </>
                : <div className="try-searching">
                        Nothing to see here ..
                        <div/>
                    Try searching for people, usernames, or keywords

                </div>
                }
            </div>
        </div>


        </div>
    )
}

export default withRouter(Payments)
