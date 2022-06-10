import React, { useEffect } from 'react'
import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'

import SqueakCard from '../../components/SqueakCard'
import Loader from '../../components/Loader'


import {
  fetchTimeline,
  selectTimelineSqueaks,
  selectTimelineSqueaksStatus,
  selectLastTimelineSqueak,
  clearTimeline,
} from '../squeaks/squeaksSlice'

import store from '../../store'


const Timeline = () => {
  const squeaks = useSelector(selectTimelineSqueaks);
  const loadingStatus = useSelector(selectTimelineSqueaksStatus)
  const lastSqueak = useSelector(selectLastTimelineSqueak)
  const dispatch = useDispatch()

  useEffect(() => {
      window.scrollTo(0, 0)
      console.log('fetchTodos');
      dispatch(clearTimeline());
      dispatch(fetchTimeline({
        limit: 10,
        lastSqueak: null,
      }));
  }, [])

  const fetchMore = () => {
    dispatch(fetchTimeline({
      limit: 10,
      lastSqueak: lastSqueak,
    }));
  }


  const renderedListItems = squeaks.map((squeak) => {
    return <SqueakCard squeak={squeak} key={squeak.getSqueakHash()} id={squeak.getSqueakHash()} user={squeak.getAuthor()} />
  })

  return <>
          <ul className="todo-list">{renderedListItems}</ul>

          {loadingStatus === 'loading' ?
          <div className="todo-list">
            <Loader />
          </div>
          :
          <div onClick={() => fetchMore()} className='squeak-btn-side squeak-btn-active'>
            LOAD MORE
          </div>
          }

         </>
}

export default Timeline
